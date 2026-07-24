use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use unjar::{Browser, cookies, cookies_for};

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
#[command(name = "unjar", version, about)]
struct Cli {
  /// Domain to export (e.g. x.com). Omit to dump every cookie
  domain: Option<String>,

  /// Browser to read from
  #[arg(short, long, default_value = "chrome")]
  browser: Browser,

  /// Output format
  #[arg(short, long, value_enum, default_value = "netscape")]
  format: Format,

  /// Write to a file instead of stdout
  #[arg(short, long)]
  output: Option<PathBuf>,
}

fn main() -> WithError<()> {
  let cli = Cli::parse();

  let jar = match &cli.domain {
    Some(domain) => cookies_for(cli.browser, domain)?,
    None => cookies(cli.browser)?,
  };

  let out = match cli.format {
    Format::Netscape => jar.to_netscape(),
    Format::Json => jar.to_json(),
    Format::Header => jar.to_header(),
  };

  match cli.output {
    Some(path) => {
      std::fs::write(&path, out)?;
      eprintln!("wrote {} cookie(s) to {}", jar.len(), path.display());
    }
    None => println!("{out}"),
  }

  Ok(())
}
