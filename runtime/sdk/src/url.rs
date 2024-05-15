use url::Url;

use crate::Result;

pub trait ToUrl {
    fn to_url(self) -> Result<Url>;
}

impl ToUrl for String {
    fn to_url(self) -> Result<Url> {
        Ok(Url::parse(&self)?)
    }
}

impl ToUrl for &str {
    fn to_url(self) -> Result<Url> {
        Ok(Url::parse(self)?)
    }
}
