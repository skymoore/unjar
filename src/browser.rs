use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// A supported browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Browser {
  Chrome,
  Chromium,
  Edge,
  Brave,
  Firefox,
  Safari,
}

/// A discovered browser profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
  browser: Browser,
  id: String,
  name: String,
  is_default: bool,
  path: PathBuf,
  cookie_db: PathBuf,
}

/// Which storage engine a browser uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
  Chromium,
  Firefox,
  Safari,
}

impl Browser {
  pub const ALL: [Browser; 6] = [
    Browser::Chrome,
    Browser::Chromium,
    Browser::Edge,
    Browser::Brave,
    Browser::Firefox,
    Browser::Safari,
  ];

  pub(crate) fn kind(&self) -> Kind {
    match self {
      Browser::Firefox => Kind::Firefox,
      Browser::Safari => Kind::Safari,
      _ => Kind::Chromium,
    }
  }

  pub(crate) fn cookie_db(&self) -> Option<PathBuf> {
    match self.kind() {
      Kind::Chromium => default_profile(*self).ok().map(|profile| profile.cookie_db),
      Kind::Firefox => firefox_db(),
      Kind::Safari => safari_db(),
    }
  }

  /// Select a profile in this browser by ID, unique display name, or path.
  pub fn find_profile(&self, selector: &str) -> Result<Profile, String> {
    let candidates: Vec<_> =
      profiles().into_iter().filter(|profile| profile.browser == *self).collect();

    match select_discovered(&candidates, selector)? {
      Some(profile) => Ok(profile),
      None if looks_like_path(selector) => Profile::from_path(*self, selector),
      None => Err(format!("{self}: profile '{selector}' not found; run `unjar list`")),
    }
  }
}

impl Profile {
  /// Build a profile from a profile directory or a cookie database.
  pub fn from_path(browser: Browser, path: impl AsRef<Path>) -> Result<Self, String> {
    if browser.kind() != Kind::Chromium {
      return Err(format!("{browser}: explicit profile paths are not supported yet"));
    }

    let path = path.as_ref();
    let (profile_path, cookie_db) = if path.is_file() {
      if path.file_name().and_then(|name| name.to_str()) != Some("Cookies") {
        return Err(format!("{}: expected a Cookies database", path.display()));
      }
      let parent = path.parent().unwrap_or(path);
      let profile = if parent.file_name().and_then(|name| name.to_str()) == Some("Network") {
        parent.parent().unwrap_or(parent)
      } else {
        parent
      };
      (profile.to_path_buf(), path.to_path_buf())
    } else {
      let db = profile_cookie_db(path).ok_or_else(|| {
        format!("{}: Cookies not found (expected Cookies or Network/Cookies)", path.display())
      })?;
      (path.to_path_buf(), db)
    };

    let id =
      profile_path.file_name().and_then(|name| name.to_str()).unwrap_or("custom").to_string();
    Ok(Self {
      browser,
      name: profile_name(&profile_path).unwrap_or_else(|| id.clone()),
      id,
      is_default: false,
      path: profile_path,
      cookie_db,
    })
  }

  pub fn browser(&self) -> Browser {
    self.browser
  }

  pub fn id(&self) -> &str {
    &self.id
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn path(&self) -> &Path {
    &self.path
  }

  pub fn cookie_db(&self) -> &Path {
    &self.cookie_db
  }

  pub fn is_default(&self) -> bool {
    self.is_default
  }
}

impl fmt::Display for Browser {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = match self {
      Browser::Chrome => "chrome",
      Browser::Chromium => "chromium",
      Browser::Edge => "edge",
      Browser::Brave => "brave",
      Browser::Firefox => "firefox",
      Browser::Safari => "safari",
    };
    f.write_str(s)
  }
}

impl FromStr for Browser {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_ascii_lowercase().as_str() {
      "chrome" => Ok(Browser::Chrome),
      "chromium" => Ok(Browser::Chromium),
      "edge" => Ok(Browser::Edge),
      "brave" => Ok(Browser::Brave),
      "firefox" | "ff" => Ok(Browser::Firefox),
      "safari" => Ok(Browser::Safari),
      other => Err(format!("unknown browser: {other}")),
    }
  }
}

/// Discover browser profiles in known locations.
pub fn profiles() -> Vec<Profile> {
  let mut profiles = Vec::new();

  for browser in [Browser::Chrome, Browser::Chromium, Browser::Edge, Browser::Brave] {
    if let Some(root) = chromium_root(browser) {
      profiles.extend(profiles_in_root(browser, &root));
    }
  }

  profiles.sort_by(|a, b| a.browser.cmp(&b.browser).then_with(|| a.id.cmp(&b.id)));
  profiles
}

/// Select a discovered profile by ID, unique display name, or known path.
pub fn find_profile(selector: &str) -> Result<Profile, String> {
  let candidates = profiles();
  match select_discovered(&candidates, selector)? {
    Some(profile) => Ok(profile),
    None if looks_like_path(selector) => {
      let path = Path::new(selector);
      let found: Vec<_> = candidates
        .iter()
        .filter(|profile| same_path(path, &profile.path) || same_path(path, &profile.cookie_db))
        .cloned()
        .collect();

      match found.as_slice() {
        [profile] => Ok(profile.clone()),
        [] if path.exists() => {
          Err(format!("cannot determine the browser for '{}'; pass --browser", path.display()))
        }
        [] => Err(format!("{}: path does not exist", path.display())),
        _ => Err(ambiguous_error(selector, &found)),
      }
    }
    None => Err(format!("profile '{selector}' not found; run `unjar list`")),
  }
}

fn select_discovered(candidates: &[Profile], selector: &str) -> Result<Option<Profile>, String> {
  let by_id: Vec<_> = candidates.iter().filter(|profile| profile.id == selector).cloned().collect();
  match by_id.as_slice() {
    [profile] => return Ok(Some(profile.clone())),
    [] => {}
    _ => return Err(ambiguous_error(selector, &by_id)),
  }

  let by_name: Vec<_> =
    candidates.iter().filter(|profile| profile.name == selector).cloned().collect();
  match by_name.as_slice() {
    [profile] => Ok(Some(profile.clone())),
    [] => Ok(None),
    _ => Err(ambiguous_error(selector, &by_name)),
  }
}

fn ambiguous_error(selector: &str, profiles: &[Profile]) -> String {
  let choices = profiles
    .iter()
    .map(|profile| format!("{} / {}", profile.browser, profile.id))
    .collect::<Vec<_>>()
    .join(", ");
  let one_browser = profiles
    .first()
    .is_some_and(|first| profiles.iter().all(|profile| profile.browser == first.browser));
  let hint = if one_browser { "use a profile ID or path" } else { "pass --browser or use a path" };
  format!("profile '{selector}' is ambiguous ({choices}); {hint}")
}

fn looks_like_path(selector: &str) -> bool {
  let path = Path::new(selector);
  path.is_absolute() || path.components().count() > 1 || path.exists()
}

fn same_path(a: &Path, b: &Path) -> bool {
  a == b
    || match (a.canonicalize(), b.canonicalize()) {
      (Ok(a), Ok(b)) => a == b,
      _ => false,
    }
}

pub(crate) fn default_profile(browser: Browser) -> Result<Profile, String> {
  let candidates: Vec<_> =
    profiles().into_iter().filter(|profile| profile.browser == browser).collect();

  if let Some(profile) = candidates.iter().find(|profile| profile.is_default) {
    return Ok(profile.clone());
  }

  match candidates.as_slice() {
    [profile] => Ok(profile.clone()),
    [] => Err(format!("{browser}: cookie database not found")),
    _ => Err(format!("{browser}: multiple profiles found; select one by name or ID")),
  }
}

fn profiles_in_root(browser: Browser, root: &Path) -> Vec<Profile> {
  let Ok(entries) = std::fs::read_dir(root) else {
    return Vec::new();
  };

  let mut profiles: Vec<_> = entries
    .filter_map(Result::ok)
    .filter_map(|entry| {
      let file_type = entry.file_type().ok()?;
      if !file_type.is_dir() || file_type.is_symlink() {
        return None;
      }
      let path = entry.path();
      let cookie_db = profile_cookie_db(&path)?;
      let directory = entry.file_name().to_string_lossy().into_owned();
      Some(Profile {
        browser,
        id: directory.clone(),
        name: profile_name(&path).unwrap_or_else(|| directory.clone()),
        is_default: false,
        path,
        cookie_db,
      })
    })
    .collect();

  let default_id = chromium_default_profile_id(root)
    .filter(|id| profiles.iter().any(|profile| profile.id == *id))
    .or_else(|| profiles.iter().any(|profile| profile.id == "Default").then(|| "Default".into()))
    .or_else(|| (profiles.len() == 1).then(|| profiles[0].id.clone()));
  if let Some(default_id) = default_id
    && let Some(profile) = profiles.iter_mut().find(|profile| profile.id == default_id)
  {
    profile.is_default = true;
  }

  profiles
}

fn profile_cookie_db(profile: &Path) -> Option<PathBuf> {
  first_existing(&[profile.join("Network").join("Cookies"), profile.join("Cookies")])
}

fn profile_name(profile: &Path) -> Option<String> {
  let data = std::fs::read_to_string(profile.join("Preferences")).ok()?;
  let value: serde_json::Value = serde_json::from_str(&data).ok()?;
  value.get("profile")?.get("name")?.as_str().filter(|name| !name.is_empty()).map(str::to_owned)
}

fn chromium_default_profile_id(root: &Path) -> Option<String> {
  let data = std::fs::read_to_string(root.join("Local State")).ok()?;
  let value: serde_json::Value = serde_json::from_str(&data).ok()?;
  value.get("profile")?.get("last_used")?.as_str().filter(|id| !id.is_empty()).map(str::to_owned)
}

/// Return the first path in `candidates` that exists.
fn first_existing(candidates: &[PathBuf]) -> Option<PathBuf> {
  candidates.iter().find(|p| p.is_file()).cloned()
}

/// Base user-data directory for a Chromium-family browser.
fn chromium_root(browser: Browser) -> Option<PathBuf> {
  #[cfg(target_os = "macos")]
  let base = dirs::config_dir()?;
  #[cfg(target_os = "linux")]
  let base = if browser == Browser::Chrome {
    std::env::var_os("CHROME_CONFIG_HOME").map(PathBuf::from).or_else(dirs::config_dir)?
  } else {
    dirs::config_dir()?
  };
  #[cfg(target_os = "windows")]
  let base = dirs::data_local_dir()?;

  #[cfg(target_os = "macos")]
  let rel: &[&str] = match browser {
    Browser::Chrome => &["Google", "Chrome"],
    Browser::Chromium => &["Chromium"],
    Browser::Edge => &["Microsoft Edge"],
    Browser::Brave => &["BraveSoftware", "Brave-Browser"],
    _ => return None,
  };
  #[cfg(target_os = "linux")]
  let rel: &[&str] = match browser {
    Browser::Chrome => &["google-chrome"],
    Browser::Chromium => &["chromium"],
    Browser::Edge => &["microsoft-edge"],
    Browser::Brave => &["BraveSoftware", "Brave-Browser"],
    _ => return None,
  };
  #[cfg(target_os = "windows")]
  let rel: &[&str] = match browser {
    Browser::Chrome => &["Google", "Chrome"],
    Browser::Chromium => &["Chromium"],
    Browser::Edge => &["Microsoft", "Edge"],
    Browser::Brave => &["BraveSoftware", "Brave-Browser"],
    _ => return None,
  };

  let mut root = base;
  for part in rel {
    root.push(part);
  }
  #[cfg(target_os = "windows")]
  root.push("User Data");

  Some(root)
}

fn firefox_db() -> Option<PathBuf> {
  #[cfg(target_os = "macos")]
  let root = dirs::config_dir()?.join("Firefox").join("Profiles");
  #[cfg(target_os = "linux")]
  let root = dirs::home_dir()?.join(".mozilla").join("firefox");
  #[cfg(target_os = "windows")]
  let root = dirs::config_dir()?.join("Mozilla").join("Firefox").join("Profiles");

  // Pick the first profile directory that contains a cookies database.
  let entries = std::fs::read_dir(&root).ok()?;
  entries.filter_map(|e| e.ok()).map(|e| e.path().join("cookies.sqlite")).find(|p| p.exists())
}

fn safari_db() -> Option<PathBuf> {
  #[cfg(target_os = "macos")]
  {
    let home = dirs::home_dir()?;
    first_existing(&[
      home.join("Library/Containers/com.apple.Safari/Data/Library/Cookies/Cookies.binarycookies"),
      home.join("Library/Cookies/Cookies.binarycookies"),
    ])
  }
  #[cfg(not(target_os = "macos"))]
  None
}

#[cfg(test)]
mod tests {
  use std::fs;

  use tempfile::tempdir;

  use super::*;

  fn profile(root: &Path, directory: &str, name: Option<&str>, network: bool) {
    let path = root.join(directory);
    let db = if network { path.join("Network").join("Cookies") } else { path.join("Cookies") };
    fs::create_dir_all(db.parent().unwrap()).unwrap();
    fs::write(db, []).unwrap();
    if let Some(name) = name {
      fs::write(path.join("Preferences"), format!(r#"{{"profile":{{"name":"{name}"}}}}"#)).unwrap();
    }
  }

  #[test]
  fn discovers_standard_profiles_with_stable_ids_and_names() {
    let dir = tempdir().unwrap();
    profile(dir.path(), "Profile 1", Some("Work"), true);
    profile(dir.path(), "Default", Some("Personal"), false);
    fs::create_dir(dir.path().join("System Profile")).unwrap();

    let mut found = profiles_in_root(Browser::Chrome, dir.path());
    found.sort_by(|a, b| a.id.cmp(&b.id));

    assert_eq!(found.len(), 2);
    assert_eq!(found[0].id(), "Default");
    assert_eq!(found[0].name(), "Personal");
    assert_eq!(found[1].id(), "Profile 1");
    assert_eq!(found[1].name(), "Work");
    assert!(found[0].is_default());
    assert!(!found[1].is_default());
    assert!(found[1].cookie_db().ends_with("Network/Cookies"));
  }

  #[test]
  fn marks_last_used_profile_as_default() {
    let dir = tempdir().unwrap();
    profile(dir.path(), "Default", Some("Personal"), false);
    profile(dir.path(), "Profile 1", Some("Work"), false);
    fs::write(dir.path().join("Local State"), r#"{"profile":{"last_used":"Profile 1"}}"#).unwrap();

    let found = profiles_in_root(Browser::Chrome, dir.path());
    assert!(found.iter().find(|profile| profile.id() == "Profile 1").unwrap().is_default());
    assert!(!found.iter().find(|profile| profile.id() == "Default").unwrap().is_default());
  }

  #[test]
  fn explicit_path_accepts_profile_directory_and_database() {
    let dir = tempdir().unwrap();
    profile(dir.path(), "custom", None, true);
    let path = dir.path().join("custom");
    let db = path.join("Network").join("Cookies");

    let from_dir = Profile::from_path(Browser::Chromium, &path).unwrap();
    let from_db = Profile::from_path(Browser::Chromium, &db).unwrap();
    assert_eq!(from_dir.cookie_db(), db);
    assert_eq!(from_db.cookie_db(), db);
    assert_eq!(from_db.path(), path);
  }

  #[test]
  fn selection_prefers_local_id_and_rejects_duplicate_names() {
    let dir = tempdir().unwrap();
    profile(dir.path(), "Default", Some("Same"), false);
    profile(dir.path(), "Profile 1", Some("Same"), false);
    let found = profiles_in_root(Browser::Chrome, dir.path());

    assert_eq!(select_discovered(&found, "Profile 1").unwrap().unwrap().id(), "Profile 1");
    let error = select_discovered(&found, "Same").unwrap_err();
    assert!(error.contains("ambiguous"));
    assert!(error.contains("chrome / Default"));
    assert!(error.contains("use a profile ID or path"));
  }

  #[test]
  fn global_selection_rejects_duplicate_ids_across_browsers() {
    let dir = tempdir().unwrap();
    profile(dir.path(), "Default", Some("Personal"), false);
    let mut found = profiles_in_root(Browser::Chrome, dir.path());
    found.extend(profiles_in_root(Browser::Brave, dir.path()));

    let error = select_discovered(&found, "Default").unwrap_err();
    assert!(error.contains("chrome / Default"));
    assert!(error.contains("brave / Default"));
    assert!(error.contains("pass --browser"));
  }
}
