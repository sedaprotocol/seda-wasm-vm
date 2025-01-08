use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Bytes, FromBytes, PromiseStatus, Result, ToBytes};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
pub struct HttpFetchAction {
    pub url:     String,
    pub options: HttpFetchOptions,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
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
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
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
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
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

    pub fn from_promise(promise_status: PromiseStatus) -> Result<Self> {
        match promise_status {
            PromiseStatus::Rejected(error) => Ok(HttpFetchResponse {
                // todo: how do i get the size from this error...?
                content_length: error.len(),
                bytes:          error,
                headers:        HashMap::default(),
                status:         0,
                url:            String::default(),
            }),
            PromiseStatus::Fulfilled(bytes) if bytes.is_none() => Ok(HttpFetchResponse {
                content_length: 0,
                bytes:          "Empty response".as_bytes().to_vec(),
                headers:        HashMap::default(),
                status:         0,
                url:            String::default(),
            }),
            _ => promise_status.parse(),
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
        Self::from_bytes(&bytes)
    }
}

impl TryFrom<Vec<u8>> for HttpFetchResponse {
    type Error = crate::SDKError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_bytes(&value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bytes_http_fetch_response() {
        let response = HttpFetchResponse {
            status:         200,
            headers:        Default::default(),
            bytes:          vec![0, 1, 2, 3],
            url:            "http://example.com".to_string(),
            content_length: 4,
        };

        let bytes = response.clone().to_bytes();
        assert_eq!(response, HttpFetchResponse::from_bytes(&bytes).unwrap());
        assert_eq!(
            response,
            HttpFetchResponse::from_bytes_vec(bytes.clone().eject()).unwrap()
        );
        assert_eq!(response, HttpFetchResponse::try_from(bytes.eject()).unwrap());
    }

    #[test]
    fn http_fetch_method_as_str() {
        assert_eq!(HttpFetchMethod::Options.as_str(), "OPTIONS");
        assert_eq!(HttpFetchMethod::Get.as_str(), "GET");
        assert_eq!(HttpFetchMethod::Post.as_str(), "POST");
        assert_eq!(HttpFetchMethod::Put.as_str(), "PUT");
        assert_eq!(HttpFetchMethod::Delete.as_str(), "DELETE");
        assert_eq!(HttpFetchMethod::Head.as_str(), "HEAD");
        assert_eq!(HttpFetchMethod::Trace.as_str(), "TRACE");
        assert_eq!(HttpFetchMethod::Connect.as_str(), "CONNECT");
        assert_eq!(HttpFetchMethod::Patch.as_str(), "PATCH");
    }

    #[test]
    fn http_fetch_response_is_ok() {
        let mut response = HttpFetchResponse {
            status:         200,
            headers:        Default::default(),
            bytes:          vec![0, 1, 2, 3],
            url:            "http://example.com".to_string(),
            content_length: 4,
        };
        assert!(response.is_ok());

        response.status = 299;
        assert!(response.is_ok());

        response.status = 199;
        assert!(!response.is_ok());

        response.status = 300;
        assert!(!response.is_ok());
    }

    #[test]
    fn http_fetch_response_from_promise() {
        let response = HttpFetchResponse {
            status:         200,
            headers:        Default::default(),
            bytes:          vec![0, 1, 2, 3],
            url:            "http://example.com".to_string(),
            content_length: 4,
        };
        let fulfilled = PromiseStatus::Fulfilled(Some(response.clone().to_bytes().eject()));
        assert_eq!(response, HttpFetchResponse::from_promise(fulfilled).unwrap());

        let fulfilled_none = PromiseStatus::Fulfilled(None);
        assert_eq!(
            HttpFetchResponse {
                status:         0,
                headers:        Default::default(),
                content_length: 0,
                bytes:          "Empty response".as_bytes().to_vec(),
                url:            String::default(),
            },
            HttpFetchResponse::from_promise(fulfilled_none).unwrap()
        );

        let error = vec![0, 1, 2, 3];
        let rejected = PromiseStatus::Rejected(error.clone());
        assert_eq!(
            HttpFetchResponse {
                status:         0,
                headers:        Default::default(),
                content_length: error.len(),
                bytes:          error,
                url:            String::default(),
            },
            HttpFetchResponse::from_promise(rejected).unwrap()
        );
    }
}
