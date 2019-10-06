use std::error::Error as StdError;
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
    #[serde(rename = "/report/rate-limit")]
    RateLimit(RateLimitException)
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
        }
    }
}

impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let exception_msg: ExceptionMsg = self.into();

        // TODO: improve
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
