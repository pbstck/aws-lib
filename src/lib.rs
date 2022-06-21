//! Simple abstraction of Aws Rust SDK.
//!
//! Provides an abstraction over often used Aws SDK.

pub mod config;
pub mod errors;
#[cfg(feature = "dynamodb")]
pub mod dynamodb;
#[cfg(feature = "lambda")]
pub mod lambda;