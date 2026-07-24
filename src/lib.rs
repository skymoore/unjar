//! Read and export cookies from local browser profiles.
//!
//! `unjar` provides access to cookies stored by locally installed browsers such
//! as Chrome, Edge, Brave, and Firefox.
//!
//! # Example
//!
//! ```no_run
//! use unjar::{Browser, cookies_for};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!   let jar = cookies_for(Browser::Chrome, "x.com")?;
//!   println!("{}", jar.to_netscape());
//!   Ok(())
//! }
//! ```

mod browser;
mod cookie;

mod chromium;
mod firefox;
mod safari;
mod sqlite;

use browser::Kind;

pub use browser::Browser;
pub use cookie::{Cookie, CookieJar};

type WithError<T> = Result<T, Box<dyn std::error::Error>>;

/// Load all cookies from the given browser.
pub fn cookies(browser: Browser) -> WithError<CookieJar> {
  let db = browser.cookie_db().ok_or_else(|| format!("{browser}: cookie database not found"))?;

  let cookies = match browser.kind() {
    Kind::Chromium => chromium::read(&db, browser)?,
    Kind::Firefox => firefox::read(&db)?,
    Kind::Safari => safari::read(&db)?,
  };

  Ok(CookieJar::new(cookies))
}

/// Load only the cookies that would be sent to `domain` (RFC 6265 domain-match).
pub fn cookies_for(browser: Browser, domain: &str) -> WithError<CookieJar> {
  Ok(cookies(browser)?.domain(domain))
}
