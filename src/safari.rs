use std::path::Path;

use crate::cookie::Cookie;

type WithError<T> = Result<T, Box<dyn std::error::Error>>;

/// Read cookies from Safari's `Cookies.binarycookies` store.
///
/// TODO: implement the binarycookies binary format parser.
pub(crate) fn read(_db: &Path) -> WithError<Vec<Cookie>> {
  Err("safari cookies (binarycookies format) parsing is not implemented yet".into())
}
