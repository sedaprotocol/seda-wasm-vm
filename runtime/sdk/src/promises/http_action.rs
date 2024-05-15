use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Bytes, FromBytes, PromiseStatus, Result, ToBytes};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HttpFetchAction {
    pub url:     String,
    pub options: HttpFetchOptions,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HttpFetchOptions {
    pub method:  HttpFetchMethod,
    pub headers: HashMap<String, String>,
    pub body:    Option<Bytes>,
}

impl Default for HttpFetchOptions {
    fn default() -> Self {
        HttpFetchOptions {
            method:  HttpFetchMethod::Get,
            headers: HashMap::new(),
            body:    None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum HttpFetchMethod {
    Options,
    Get,
    Post,
    Put,
    Delete,
    Head,
    Trace,
    Connect,
    Patch,
}

impl HttpFetchMethod {
    pub fn as_str(&self) -> &str {
        match self {
            HttpFetchMethod::Options => "OPTIONS",
            HttpFetchMethod::Get => "GET",
            HttpFetchMethod::Post => "POST",
            HttpFetchMethod::Put => "PUT",
            HttpFetchMethod::Delete => "DELETE",
            HttpFetchMethod::Head => "HEAD",
            HttpFetchMethod::Trace => "TRACE",
            HttpFetchMethod::Connect => "CONNECT",
            HttpFetchMethod::Patch => "PATCH",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HttpFetchResponse {
    /// HTTP Status code
    pub status: u16,

    /// Response headers
    pub headers: HashMap<String, String>,

    /// Response body in bytes
    pub bytes: Vec<u8>,

    /// The final URL that was resolved
    pub url: String,

    /// The byte length of the response
    pub content_length: usize,
}

impl HttpFetchResponse {
    pub fn is_ok(&self) -> bool {
        self.status >= 200 && self.status <= 299
    }

    pub fn from_promise(promise_status: PromiseStatus) -> Self {
        match promise_status {
            PromiseStatus::Rejected(error) => HttpFetchResponse {
                // todo: how do i get the size from this error...?
                content_length: error.len(),
                bytes:          error,
                headers:        HashMap::default(),
                status:         0,
                url:            String::default(),
            },
            _ => promise_status.parse().unwrap(),
        }
    }
}

impl ToBytes for HttpFetchResponse {
    fn to_bytes(self) -> Bytes {
        serde_json::to_vec(&self).unwrap().to_bytes()
    }
}

impl FromBytes for HttpFetchResponse {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(Into::into)
    }

    fn from_bytes_vec(bytes: Vec<u8>) -> Result<Self> {
        serde_json::from_slice(&bytes).map_err(Into::into)
    }
}

impl TryFrom<Vec<u8>> for HttpFetchResponse {
    type Error = serde_json::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        serde_json::from_slice(&value)
    }
}
