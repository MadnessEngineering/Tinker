use std::error::Error as StdError;
use std::fmt;
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StdError for Error {}

impl<T: StdError> From<T> for Error {
    fn from(err: T) -> Self {
        Error::new(err.to_string())
    }
}

pub fn ensure_url_scheme(url: &str) -> String {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        format!("https://{}", url)
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_url_scheme() {
        assert_eq!(ensure_url_scheme("example.com"), "https://example.com");
        assert_eq!(
            ensure_url_scheme("http://example.com"),
            "http://example.com"
        );
        assert_eq!(
            ensure_url_scheme("https://example.com"),
            "https://example.com"
        );
    }
} 