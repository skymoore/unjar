use std::path::Path;

use crate::Result;
use crate::cookie::Cookie;

/// Read cookies from Safari's `Cookies.binarycookies` store.
///
/// TODO: implement the binarycookies binary format parser.
pub(crate) fn read(_db: &Path) -> Result<Vec<Cookie>> {
  Err("safari cookies (binarycookies format) parsing is not implemented yet".into())
}
