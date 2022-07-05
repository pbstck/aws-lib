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
#[cfg(feature = "dynamodb")]
pub extern crate aws_sdk_dynamodb;
#[cfg(feature = "lambda")]
pub mod lambda;
#[cfg(feature = "lambda")]
pub extern crate aws_sdk_lambda;
pub mod log;

#[cfg(feature = "ecs")]
pub mod ecs;
#[cfg(feature = "ecs")]
pub extern crate aws_sdk_ecs;
