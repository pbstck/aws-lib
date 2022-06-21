//! Simple abstraction of Aws Rust SDK.
//!
//! Provides an abstraction over often used Aws SDK.

pub mod config;
pub mod errors;
//Export crate for re-use in project
pub extern crate aws_config;
pub extern crate aws_types;

#[cfg(feature = "dynamodb")]
pub mod dynamodb;
pub extern crate aws_sdk_dynamodb;