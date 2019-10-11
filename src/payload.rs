use std::collections::HashMap;

use erased_serde::Serialize as ErasedSerialize;
use warp::http::StatusCode;

pub struct Response {
    value: Option<Box<dyn ErasedSerialize + Send>>,
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

pub struct ResponseBuilder {
    status_code: StatusCode,
    headers: HashMap<String, String>,
}

impl ResponseBuilder {
    /// Create a new payload response builder with the given status code.
    pub fn new(status_code: StatusCode) -> Self {
        ResponseBuilder {
            status_code,
            headers: HashMap::new(),
        }
    }

    /// Create a response with a 201 Created status code.
    pub fn created() -> Self {
        Self::new(StatusCode::CREATED)
    }

    /// Set a response header.
    pub fn header(mut self, header_name: String, header_value: String) -> Self {
        self.headers.insert(header_name, header_value);
        self
    }

    /// Add a (relative) next-page URI header to the response.
    pub fn next_page_uri(mut self, uri: String) -> Self {
        self.headers.insert("x-next".to_owned(), uri);
        self
    }

    /// Add a Location URI header. Only makes sense with the Created or a Redirection status.
    pub fn content_uri(mut self, uri: String) -> Self {
        self.headers.insert("Location".to_owned(), uri);
        self
    }
    /// Build an empty response.
    pub fn empty(self) -> Response {
        Response {
            value: None,
            status_code: self.status_code,
            headers: self.headers,
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
