use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand, ValueEnum};
use unjar::{
  Browser, CookieJar, Profile, cookies, cookies_for, cookies_for_profile, cookies_from,
  find_profile, profiles,
};

type WithError<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
  /// Netscape cookies.txt (curl, wget, yt-dlp)
  Netscape,
  /// JSON array
  Json,
  /// Cookie header value (k=v; k2=v2)
  Header,
}

/// Read and export cookies from local browser profiles.
#[derive(Debug, Parser)]
#[command(name = "unjar", version, about, subcommand_precedence_over_arg = true)]
struct Cli {
  #[command(subcommand)]
  command: Option<Command>,

  #[command(flatten)]
  export: Export,
}

#[derive(Debug, Subcommand)]
enum Command {
  /// List discovered browser profiles
  List,
}

#[derive(Debug, Args)]
struct Export {
  /// Domain to export (e.g. x.com). Omit to dump every cookie
  domain: Option<String>,

  /// Browser to read from
  #[arg(short, long, default_value = "chrome")]
  browser: Browser,

  /// Profile name, discovered ID, profile directory, or Cookies database
  #[arg(short, long)]
  profile: Option<String>,

  /// Output format
  #[arg(short, long, value_enum, default_value = "netscape")]
  format: Format,

  /// Write to a file instead of stdout
  #[arg(short, long)]
  output: Option<PathBuf>,
}

fn main() -> WithError<()> {
  let cli = Cli::parse();

  if matches!(cli.command, Some(Command::List)) {
    print_profiles();
    return Ok(());
  }

  export(cli.export)
}

fn print_profiles() {
  println!("BROWSER\tID\tNAME\tPATH");
  for profile in profiles() {
    println!(
      "{}\t{}\t{}\t{}",
      profile.browser(),
      profile.id(),
      profile.name(),
      profile.path().display()
    );
  }
}

fn export(args: Export) -> WithError<()> {
  let profile = match &args.profile {
    Some(selector) => Some(select_profile(args.browser, selector)?),
    None => None,
  };

  let jar = match (profile, &args.domain) {
    (Some(profile), Some(domain)) => cookies_for_profile(&profile, domain)?,
    (Some(profile), None) => cookies_from(&profile)?,
    (None, Some(domain)) => cookies_for(args.browser, domain)?,
    (None, None) => cookies(args.browser)?,
  };

  let out = format_jar(&jar, args.format);

  match args.output {
    Some(path) => {
      std::fs::write(&path, out)?;
      eprintln!("wrote {} cookie(s) to {}", jar.len(), path.display());
    }
    None => println!("{out}"),
  }

  Ok(())
}

fn select_profile(browser: Browser, selector: &str) -> Result<Profile, String> {
  let discovered = find_profile(browser, selector);
  if discovered.is_ok() {
    return discovered;
  }

  let path = Path::new(selector);
  if path.is_absolute() || path.components().count() > 1 || path.exists() {
    Profile::from_path(browser, path)
  } else {
    discovered
  }
}

fn format_jar(jar: &CookieJar, format: Format) -> String {
  match format {
    Format::Netscape => jar.to_netscape(),
    Format::Json => jar.to_json(),
    Format::Header => jar.to_header(),
  }
}

#[cfg(test)]
mod tests {
  use clap::Parser;

  use super::*;

  #[test]
  fn list_is_a_subcommand() {
    let cli = Cli::try_parse_from(["unjar", "list"]).unwrap();
    assert!(matches!(cli.command, Some(Command::List)));
  }

  #[test]
  fn legacy_export_syntax_and_profile_are_accepted() {
    let cli = Cli::try_parse_from([
      "unjar",
      "example.com",
      "--browser",
      "chrome",
      "--profile",
      "chrome:Profile 1",
    ])
    .unwrap();
    assert_eq!(cli.export.domain.as_deref(), Some("example.com"));
    assert_eq!(cli.export.browser, Browser::Chrome);
    assert_eq!(cli.export.profile.as_deref(), Some("chrome:Profile 1"));
  }
}
