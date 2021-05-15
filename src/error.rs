pub use anyhow::{Context, Result as AnyResult};
use nom::{error::{ErrorKind, FromExternalError, ParseError}};

#[macro_export]
macro_rules! elog {
    ($msg:literal $(,)?) => {
        anyhow::anyhow!(format!("[{}].[{}]: {}", file!(), line!(), $msg))
    };
    ($fmt:expr, $($arg:tt)*) => {
        anyhow::anyhow!(format!("[{}].[{}]: {}", file!(), line!(), format!($fmt, $($arg)*)))
    };
}

#[derive(Debug)]
pub enum WikitError {
    Anyhow(anyhow::Error),
}

impl<I> ParseError<I> for WikitError {
  fn from_error_kind(_input: I, kind: ErrorKind) -> Self {
    WikitError::Anyhow(anyhow::anyhow!(format!("{}", kind.description())))
  }

  fn append(_: I, _: ErrorKind, other: Self) -> Self {
    other
  }
}

impl<I> FromExternalError<I, anyhow::Error> for WikitError {
    fn from_external_error(_input: I, _kind: ErrorKind, e: anyhow::Error) -> Self {
        WikitError::Anyhow(e)
    }
}
