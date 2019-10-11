use diesel::result::Error as DieselError;
use failure::{Fail, Context, Backtrace};
use failure::Error as FailureError;

use std::fmt::{self, Display};

/// convenience alias wrapper Result.
pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Clone, Fail)]
pub enum ErrorKind {
    #[fail(display="From Failure")]
    FromFailure,
    #[fail(display = "{}", _0)]
    DatabaseError(String),
    /// Document not found in database.  Results in status code 404
    #[fail(display = "The resource ({}) requested could not be found in database", _0)]
    NotFound{
        type_name: String
    },
    /// The key used already exists in the database. Results in status code 402.
    #[fail(display = "{}", _0)]
    AlreadyExists(String),
    #[fail(display = "{}", _0)]
    InternalServerError(String),
}

#[derive(Debug)]
pub struct Error {
    /// Inner `Context` with the `Fail` implementor.
    pub(crate) inner: Context<ErrorKind>,
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
    pub fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
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
    fn from(error: FailureError) -> Error {
        Error { inner: error.context(ErrorKind::FromFailure) }
    }
}

impl From<diesel::result::Error> for Error {
    fn from(error: DieselError) -> Error {
        use diesel::result::DatabaseErrorKind;

        match error {
            diesel::result::Error::DatabaseError(err, _) => {
                let err = match err {
                    DatabaseErrorKind::ForeignKeyViolation => {
                        "A foreign key constraint was violated in the database"
                    }
                    DatabaseErrorKind::SerializationFailure => {
                        "Value failed to serialize in the database"
                    }

                    DatabaseErrorKind::UnableToSendCommand => {
                        "Database protocol violation, possibly too many bound parameters"
                    }

                    DatabaseErrorKind::UniqueViolation => {
                        "A unique constraint was violated in the database"
                    }

                    DatabaseErrorKind::__Unknown => {
                        "An unknwon error occurred in the database"
                    }
                }
                .to_string();
            Error::from(ErrorKind::DatabaseError(err))

            }
            diesel::result::Error::NotFound => Error::from(ErrorKind::NotFound {
                type_name: "Not implemented".to_string(),
            }),
            err => {
                log::error!("unhandled database error: '{}'", err);
                Error::from(ErrorKind::InternalServerError(
                        format!("Internal Server Error")))
            }
        }
    }
}
