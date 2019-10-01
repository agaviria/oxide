use std::{
	env,
	fmt::{Display, Formatter, Result as FmtResult},
};
use failure::{Backtrace, Context, Fail};

/// convenience alias wrapper Result.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Sentry package error kind.
#[derive(Debug, Fail)]
pub enum ErrorKind {
	#[fail(display = "Hasher config error")]
	HashConfigError(argonautica::Error),

	/// An error with an arbitrary message, referenced as &'static str
	#[fail(display = "{}", _0)]
	Message(&'static str),

	/// An error with an arbitrary message, stored as String
	#[fail(display = "{}", _0)]
	Msg(String),

	#[fail(display = "Base64 encode error")]
	EnvVarEncoder(argonautica::Error),

	#[fail(display = "Failure error")]
	FromFailure,

	#[fail(display = "I/O error")]
	IO,

	#[fail(display = "Hash error")]
	Hasher,

	#[fail(display = "Invalid Vector length: got {}, expected {}", got, expected)]
	VecLength { got: usize, expected: usize },
}

/// Sentry application error.
#[derive(Debug)]
pub struct Error {
	inner: Context<ErrorKind>,
}

impl Error {
	/// Returns the error variant and contents.
	pub fn kind(&self) -> &ErrorKind {
		self.inner.get_context()
	}

	/// Returns the immediate cause of error (e.g. the next error in the chain)
	pub fn cause(&self) -> Option<&dyn Fail> {
		self.inner.cause()
	}

	pub fn backtrace(&self) -> Option<&Backtrace> {
		self.inner.backtrace()
	}

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
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let show_trace = match env::var("RUST_BACKTRACE") {
			Ok(r) => {
				if r == "1" {
					true
				} else {
					false
				}
			}
			Err(_) => false,
		};

		let backtrace = match self.backtrace() {
			Some(b) => format!("{}", b),
			None => String::from("Unknown"),
		};

		let trace_fmt = format!("\nBacktrace: {:?}", backtrace);
		let inner_fmt = format!("{}", self.inner);
		let mut print_format = inner_fmt.clone();
		if show_trace {
			print_format.push_str(&trace_fmt);
		}
		Display::fmt(&print_format, f)
	}
}

impl<E: Into<ErrorKind>> From<E> for Error {
	fn from(err: E) -> Error {
		Context::new(err.into()).into()
	}
}

impl From<Context<ErrorKind>> for Error {
	fn from(inner: Context<ErrorKind>) -> Error {
		Error { inner: inner }
	}
}

impl From<&'static str> for Error {
	fn from(msg: &'static str) -> Error {
		ErrorKind::Message(msg).into()
	}
}

impl From<String> for Error {
	fn from(msg: String) -> Error {
		ErrorKind::Msg(msg).into()
	}
}

impl From<failure::Error> for Error {
	fn from(err: failure::Error) -> Error {
		Error { inner: err.context(ErrorKind::FromFailure) }
	}
}

impl From<::std::io::Error> for Error {
	fn from(err: ::std::io::Error) -> Error {
		Error { inner: err.context(ErrorKind::IO) }
	}
}

impl From<argonautica::Error> for Error {
	fn from(err: argonautica::Error) -> Error {
		Error { inner: err.context(ErrorKind::Hasher) }
	}
}

/// ParseError handles the parse validation errors for HashVersion V1.
#[derive(Debug)]
pub enum ParseError {
	/// base64 decode error
	DecodeError(base64::DecodeError),
	/// Utf-8 string error
	Utf8(std::str::Utf8Error),
	/// vector length validator
	InvalidVecLen,
	/// slice validation
	InvalidSlice,
	/// byte size validation can occur in  `salt` or  `hash` types of V1Hash
	InvalidLen,
}

impl Display for ParseError {
	fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
		match &*self {
			ParseError::DecodeError(e) => write!(fmt, "Decode error: {}", e),
			ParseError::InvalidVecLen => write!(fmt, "Invalid vector"),
			ParseError::InvalidSlice => write!(fmt, "Invalid Slice"),
			ParseError::InvalidLen => write!(fmt, "Invalid byte size"),
			ParseError::Utf8(e) => write!(fmt, "Utf-8 error: {}", e),
		}
	}
}

impl PartialEq for ParseError {
	fn eq(&self, other: &Self) -> bool {
		match self {
			ParseError::InvalidVecLen => match other {
				ParseError::InvalidVecLen => true,
				_ => false,
			},
			ParseError::InvalidSlice => match other {
				ParseError::InvalidSlice => true,
				_ => false,
			},
			ParseError::InvalidLen => match other {
				ParseError::InvalidLen => true,
				_ => false,
			},
			ParseError::DecodeError(_) => false,
			ParseError::Utf8(_) => false,
		}
	}
}

impl From<base64::DecodeError> for ParseError {
	fn from(err: base64::DecodeError) -> ParseError {
		ParseError::DecodeError(err)
	}
}

// #[macro_export]
// /// validates ParseError Eq implementation
// macro_rules! validate {
//	($cond:expr, $e:expr) => {
//		if !($cond) {
//			return Err($e);
//		}
//	};
//	($cond:expr, $fmt:expr, $($arg:tt)+) => {
//		if !($cond) {
//			return Err($fmt, $($arg)+);
//		}
//	};
// }
