use std::{
	error::Error as StdError,
	collections::HashMap,
	borrow::{Borrow, Cow},
};
use serde::{Deserialize, Serialize};

pub const INTERNAL_SERVER_ERROR: Fault = Fault::Static(StaticException::InternalServerError);
pub const NOT_FOUND: Fault = Fault::Static(StaticException::NotFound);

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Fault {
	/// Static will handle exceptions related to the static server
	#[serde(rename = "about:blank")]
	Static(StaticException),
	/// RateLimit handles exceptions related to filter gated request behind a
	/// leaky bucket rate limiter
	#[serde(rename = "/report-exception/rate-limit")]
	RateLimit(RateLimitException),
	#[serde(rename = "/report-excetion/payload-too-large")]
	#[serde(rename_all = "camelCase")]
	PayloadTooLarge { limit: u64 },
	#[serde(rename = "/report-exception/invalid-json")]
	#[serde(rename_all = "camelCase")]
	InvalidJson {
		category: JsonDeserializeError,
	},
	#[serde(rename = "/report-exception/invalid-params")]
	#[serde(rename_all = "camelCase")]
	InvalidParams {
		invalid_params: InvalidParams
	}
}

impl Fault {
	pub fn to_status_code(&self) -> warp::http::StatusCode {
		use warp::http::StatusCode;
		use Fault::*;

		match self {
			Static(StaticException::NotFound) => StatusCode::NOT_FOUND,
			Static(StaticException::InternalServerError) => {
				StatusCode::INTERNAL_SERVER_ERROR
			},
			RateLimit(_) =>  warp::http::StatusCode::TOO_MANY_REQUESTS,
			PayloadTooLarge { .. } => warp::http::StatusCode::PAYLOAD_TOO_LARGE,
			InvalidJson { .. } => warp::http::StatusCode::BAD_REQUEST,
			InvalidParams { .. } => warp::http::StatusCode::BAD_REQUEST,
		}
	}
}

impl std::fmt::Display for Fault {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let exception_msg: ExceptionMsg = self.into();

		f.write_str(&format!("{:?}", exception_msg))
	}
}

impl StdError for Fault {}

/// Serde serializes an ExceptionMsg with its title field set to None and
/// with its problem field to the Fault::Static variant, the title field in the
/// generated serialization is taken from the StaticException variant.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "title")]
pub enum StaticException {
	#[serde(rename = "Not Found")]
	NotFound,

	#[serde(rename = "Internal Server Error")]
	InternalServerError,
}

#[derive(Debug, Serialize)]
pub struct ExceptionMsg<'a> {
	#[serde(flatten)]
	pub fault: &'a Fault,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub title: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub status: Option<u16>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub detail: Option<String>,
}

impl<'a> From<&'a Fault> for ExceptionMsg<'a> {
	fn from(fault: &'a Fault) -> ExceptionMsg<'a> {
		use Fault::*;

		let status = Some(fault.to_status_code().as_u16());

		let (title, detail) = match fault {
			Static(_) => {
				(None, None)
			}

			RateLimit(_) => {
				(
					Some("Your request has been rate limited.".to_owned()),
					None,
				)
			}

			PayloadTooLarge { limit } => {
				(
					Some("Your request payload exceeds max length limit.".to_owned()),
					Some(format!("The request payload limit was {} bytes.", limit)),
				)
			}

			InvalidJson { .. } => {
				(
					Some("Your request JSON was malformed.".to_owned()),
					Some("The JSON might be syntactically incorrect, or it does not adhere to the endpoint's schema. Refer to the JSON category for more information.".to_owned()),
				)
			}
			InvalidParams { .. } => {
				(
					Some("Your request parameters are not valid.".to_owned()),
					None,
				)
			}
		};

		ExceptionMsg {
			fault,
			title,
			status,
			detail,
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitException {
	pub wait_time_millis: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthenticationTokenProblemCategory {
	Missing,
	Malformed,
	Expired,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvalidParams {
	#[serde(flatten)]
	inner: HashMap<Cow<'static, str>, Vec<InvalidParamsReason>>,
}

impl InvalidParams {
	pub fn new() -> Self {
		Self {
			inner: HashMap::new(),
		}
	}

	pub fn is_empty(&self) -> bool {
		!self.inner.iter().any(|(_, reasons)| !reasons.is_empty())
	}

	pub fn add<S: Into<Cow<'static, str>>>(
		&mut self,
		parameter: S,
		reason: InvalidParamsReason,
	) {
		self.inner
			.entry(parameter.into())
			.or_insert(vec![])
			.push(reason)
	}
}

impl<E: Borrow<validator::ValidationErrors>> From<E> for InvalidParams {
	fn from(validation_errors: E) -> InvalidParams {
		// heck handles Serde's rename_all "camelCase"
		use heck::MixedCase;
		let mut invalid_params = InvalidParams::new();

		for (field, errors) in validation_errors
			.borrow()
				.field_errors()
				.into_iter() {
					for error in errors.into_iter() {
						invalid_params.add(field.to_mixed_case(), error.into())
					}
				}

		invalid_params
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InvalidParamsReason {
	MustBeEmailAddress,
	MustBeUrl,
	MustBeInRange { min: f64, max: f64 },
	MustHaveLengthBetween {
		#[serde(skip_serializing_if = "Option::is_none")]
		min: Option<u64>,

		#[serde(skip_serializing_if = "Option::is_none")]
		max: Option<u64>
	},
	MustHaveLengthExactly { length: u64 },
	AlreadyExists,
	InvalidToken { category: AuthenticationTokenProblemCategory },
	Other,
}

impl<E: Borrow<validator::ValidationError>> From<E> for InvalidParamsReason {
	fn from(validation_error: E) -> InvalidParamsReason {
		use InvalidParamsReason::*;

		let validation_error: &validator::ValidationError = validation_error.borrow();

		match validation_error.code.as_ref() {
			"email" => MustBeEmailAddress,
			"url" => MustBeUrl,
			"range" => {
				let min: Option<f64> = validation_error
					.params
					.get("min")
					.map(|v| v.as_f64().unwrap());
				let max: Option<f64> = validation_error
					.params
					.get("max")
					.map(|v| v.as_f64().unwrap());

				match (min, max) {
					(Some(min), Some(max)) => MustBeInRange { min, max },
					_ => Other,
				}
			}
			"length" => {
				let min: Option<u64> = validation_error
					.params
					.get("min")
					.map(|v| v.as_u64().unwrap());
				let max: Option<u64> = validation_error
					.params
					.get("max")
					.map(|v| v.as_u64().unwrap());
				let equal: Option<u64> = validation_error
					.params
					.get("equal")
					.map(|v| v.as_u64().unwrap());

				match (min, max, equal) {
					(min@Some(_), max, None) => MustHaveLengthBetween { min, max },
					(min, max@Some(_), None) => MustHaveLengthBetween { min, max },
					(None, None, Some(equal)) => MustHaveLengthExactly { length: equal },
					_ => Other,
				}
			}
			_ => Other,
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JsonDeserializeError {
	Syntactic,
	Semantic,
	PrematureEnd,
	Other,
}

impl From<serde_json::error::Category> for JsonDeserializeError {
	fn from(category: serde_json::error::Category) -> Self {
		use serde_json::error::Category::*;

		match category {
			Syntax => Self::Syntactic,
			Data => Self::Semantic,
			Eof => Self::PrematureEnd,
			_ => Self::Other,
		}
	}
}

impl From<&serde_json::Error> for JsonDeserializeError {
	fn from(error: &serde_json::Error) -> Self {
		error.classify().into()
	}
}
