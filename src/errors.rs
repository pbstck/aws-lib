use serde::Serialize;
use std::fmt;

/// Default retry count used in the clients
pub const DEFAULT_RETRY_COUNT: u32 = 5;

#[derive(Debug, Serialize)]
pub struct AwsError {
    details: String,
}

impl std::error::Error for AwsError {
    // this implementation required `Debug` and `Display` traits
}

impl fmt::Display for AwsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl AwsError {
    pub fn new(msg: &str) -> AwsError {
        AwsError {
            details: msg.to_string(),
        }
    }
}
