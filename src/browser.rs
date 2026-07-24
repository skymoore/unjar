use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

/// A supported browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Browser {
  Chrome,
  Chromium,
  Edge,
  Brave,
  Firefox,
  Safari,
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

  /// Path to the cookie database for this browser, if it exists on disk.
  pub fn cookie_db(&self) -> Option<PathBuf> {
    match self.kind() {
      Kind::Chromium => chromium_db(self),
      Kind::Firefox => firefox_db(),
      Kind::Safari => safari_db(),
    }
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

/// Return the first path in `candidates` that exists.
fn first_existing(candidates: &[PathBuf]) -> Option<PathBuf> {
  candidates.iter().find(|p| p.exists()).cloned()
}

/// Base "User Data" directory for a chromium-family browser.
fn chromium_root(browser: &Browser) -> Option<PathBuf> {
  #[cfg(target_os = "macos")]
  let base = dirs::config_dir()?; // ~/Library/Application Support
  #[cfg(target_os = "linux")]
  let base = dirs::config_dir()?; // ~/.config
  #[cfg(target_os = "windows")]
  let base = dirs::data_local_dir()?; // %LOCALAPPDATA%

  let rel: &[&str] = match (browser, cfg!(target_os = "macos")) {
    (Browser::Chrome, true) => &["Google", "Chrome"],
    (Browser::Chrome, false) => &["google-chrome"],
    (Browser::Chromium, true) => &["Chromium"],
    (Browser::Chromium, false) => &["chromium"],
    (Browser::Edge, true) => &["Microsoft Edge"],
    (Browser::Edge, false) => &["microsoft-edge"],
    (Browser::Brave, _) => &["BraveSoftware", "Brave-Browser"],
    _ => return None,
  };

  // On Windows chromium stores everything under a "User Data" folder.
  let mut root = base;
  for part in rel {
    root.push(part);
  }
  #[cfg(target_os = "windows")]
  root.push("User Data");

  Some(root)
}

fn chromium_db(browser: &Browser) -> Option<PathBuf> {
  let root = chromium_root(browser)?;
  let profile = root.join("Default");
  // Newer chromium keeps cookies under Network/, older versions at the profile root.
  first_existing(&[profile.join("Network").join("Cookies"), profile.join("Cookies")])
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
