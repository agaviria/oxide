use failure::{Fail, Context, Backtrace};
use failure::Error as FailureError;

use std::fmt::{self, Display};

/// convenience alias wrapper Result.
pub type Result<T> = ::std::result::Result<T, Error>;

/// The error type used in `libyobicash`.
#[derive(Debug)]
pub struct Error {
    /// Inner `Context` with the `Fail` implementor.
    inner: Context<ErrorKind>,
}


/// The different types of errors used in `libyobicash`.
#[derive(Debug, Clone, Copy, Fail)]
pub enum ErrorKind {
    #[fail(display="From Failure")]
    FromFailure,
    // #[fail(display="Magic Crypt failure: {:?}", _0)]
    // MagicCryptError(CryptError),
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}
impl Error {
    pub fn kind(&self) -> ErrorKind {
        *self.inner.get_context()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error { inner: Context::new(kind) }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner: inner }
    }
}

impl From<FailureError> for Error {
    fn from(e: FailureError) -> Error {
        Error { inner: e.context(ErrorKind::FromFailure) }
    }
}
