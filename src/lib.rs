//! A framework-independent Aptabase client for Rust applications.

mod client;
mod config;
mod dispatcher;
mod sys;

pub use client::AptabaseClient;
pub use config::InitOptions;
