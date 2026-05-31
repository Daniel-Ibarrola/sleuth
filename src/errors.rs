use std::fmt;

#[derive(Debug)]
pub enum SleuthError {
    InvalidFormat(String),
    GithubError(octocrab::Error),
    RequestError(reqwest::Error),
}

impl fmt::Display for SleuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SleuthError::InvalidFormat(message) => write!(f, "{message}"),
            SleuthError::GithubError(error) => write!(f, "GitHub API error: {error}"),
            SleuthError::RequestError(error) => write!(f, "Request error: {error}"),
        }
    }
}

impl std::error::Error for SleuthError {}

impl From<octocrab::Error> for SleuthError {
    fn from(error: octocrab::Error) -> Self {
        SleuthError::GithubError(error)
    }
}

impl From<reqwest::Error> for SleuthError {
    fn from(error: reqwest::Error) -> Self {
        SleuthError::RequestError(error)
    }
}
