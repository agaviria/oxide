use std::collections::HashMap;

use erased_serde::Serialize as ErasedSerialize;
use warp::http::StatusCode;

pub struct Response {
    value: Option<Box<dyn ErasedSerialize + Send>>,
    status_code: StatusCode,
    headers: HashMap<String, String>,
}

pub struct ResponseBuilder {
    status_code: StatusCode,
    headers: HashMap<String, String>,
}

impl Response {
    /// The response headers.
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// The response status code.
    pub fn status_code(&self) -> StatusCode {
        self.status_code
    }

    /// The response value.
    pub fn value(&self) -> &Option<Box<dyn ErasedSerialize + Send>> {
        &self.value
    }
}

impl ResponseBuilder {
    /// Create a new payload response builder with the given status code.
    pub fn new(status_code: StatusCode) -> Self {
        ResponseBuilder {
            status_code,
            headers: HashMap::new(),
        }
    }

    /// Build the payload response with the given value.
    pub fn body<T>(self, value: T) -> Response
        where
        T: ErasedSerialize + Send + 'static,
        {
            Response {
                value: Some(Box::new(value) as Box<dyn ErasedSerialize + Send>),
                status_code: self.status_code,
                headers: self.headers,
            }
        }

    /// Create a response with a 200 OK status code.
    pub fn ok() -> Self {
        Self::new(StatusCode::OK)
    }
}
