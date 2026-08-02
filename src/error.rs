use std::fmt;

/// An error returned by unjar.
#[derive(thiserror::Error)]
#[non_exhaustive]
pub enum Error {
  /// A user-facing error without a lower-level source.
  #[error("{0}")]
  Message(String),
  /// An I/O operation failed.
  #[error(transparent)]
  Io(#[from] std::io::Error),
  /// A browser database operation failed.
  #[error(transparent)]
  Database(#[from] rusqlite::Error),
  /// Text returned by a platform service was not valid UTF-8.
  #[error(transparent)]
  Utf8(#[from] std::string::FromUtf8Error),
}

impl fmt::Debug for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Display::fmt(self, f)
  }
}

impl From<String> for Error {
  fn from(message: String) -> Self {
    Self::Message(message)
  }
}

impl From<&str> for Error {
  fn from(message: &str) -> Self {
    message.to_owned().into()
  }
}

/// A result returned by unjar.
pub type Result<T> = std::result::Result<T, Error>;
