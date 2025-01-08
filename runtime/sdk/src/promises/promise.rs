use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use super::PromiseAction;
use crate::ToBytes;

// TODO: Fulfilled and Rejected could now just be our Bytes type.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
pub enum PromiseStatus {
    /// The promise completed
    Fulfilled(Option<Vec<u8>>),

    /// There was an error executing this promise
    // TODO: Is there ever a case where Rejected isn't a string?
    // HTTP rejections could be an object(but encoded in a string).
    // Could private the type and then have methods or something.
    Rejected(Vec<u8>),
}

impl PromiseStatus {
    /// Helper function that immidiatly assumes that the promise has been
    /// fulfilled and return the value. Panics if result is not fulfilled
    pub fn fulfilled(self) -> Vec<u8> {
        if let Self::Fulfilled(Some(value)) = self {
            return value;
        }

        panic!("Promise is not fulfilled: {:?}", &self);
    }

    pub fn parse<T>(self) -> Result<T, T::Error>
    where
        T: TryFrom<Vec<u8>>,
        T: TryFrom<Vec<u8>, Error = crate::SDKError>,
    {
        let value = self.fulfilled();

        value.try_into()
    }
}

impl<T: crate::ToBytes, E: std::error::Error> From<Result<T, E>> for PromiseStatus {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(fulfilled) => PromiseStatus::Fulfilled(Some(fulfilled.to_bytes().eject())),
            Err(rejection) => PromiseStatus::Rejected(rejection.to_string().to_bytes().eject()),
        }
    }
}

impl<T: crate::ToBytes, E: std::error::Error> From<Result<Option<T>, E>> for PromiseStatus {
    fn from(value: Result<Option<T>, E>) -> Self {
        match value {
            Ok(fulfilled) => PromiseStatus::Fulfilled(fulfilled.map(|inner| inner.to_bytes().eject())),
            Err(rejection) => PromiseStatus::Rejected(rejection.to_string().to_bytes().eject()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
pub struct Promise {
    /// The name of the action we should execute
    pub action: PromiseAction,

    /// The status of the promise, will include the result if it's fulfilled
    pub status: PromiseStatus,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::SDKError;

    #[test]
    #[should_panic]
    fn test_promise_status_panics_if_rejected() {
        let status = PromiseStatus::Rejected("error".to_string().into_bytes());
        status.fulfilled();
    }

    #[test]
    #[should_panic]
    fn test_promise_status_panics_if_none() {
        let status = PromiseStatus::Fulfilled(None);
        status.fulfilled();
    }

    #[test]
    fn promise_status_from_result() {
        let success: Result<String, SDKError> = Ok("hello".to_string());
        let promise_status: PromiseStatus = success.into();
        assert_eq!(
            promise_status,
            PromiseStatus::Fulfilled(Some("hello".to_string().into_bytes()))
        );

        let error: Result<String, SDKError> = Err(SDKError::InvalidValue);
        let promise_status: PromiseStatus = error.into();
        assert_eq!(
            promise_status,
            PromiseStatus::Rejected(SDKError::InvalidValue.to_string().into_bytes())
        );
    }

    #[test]
    fn promise_status_from_result_option() {
        let success_some: Result<Option<String>, SDKError> = Ok(Some("hello".to_string()));
        let promise_status: PromiseStatus = success_some.into();
        assert_eq!(
            promise_status,
            PromiseStatus::Fulfilled(Some("hello".to_string().into_bytes()))
        );

        let success_none: Result<Option<String>, SDKError> = Ok(None);
        let promise_status: PromiseStatus = success_none.into();
        assert_eq!(promise_status, PromiseStatus::Fulfilled(None));

        let error: Result<Option<String>, SDKError> = Err(SDKError::InvalidValue);
        let promise_status: PromiseStatus = error.into();
        assert_eq!(
            promise_status,
            PromiseStatus::Rejected(SDKError::InvalidValue.to_string().into_bytes())
        );
    }
}
