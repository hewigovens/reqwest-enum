//! `reqwest-enum` simplifies making HTTP(S) requests by providing a structured way to define API targets (`target::Target` trait), manage request providers (`provider::Provider`), and handle authentication (`http::AuthMethod`).
//!
//! # Key Features
//!
//! *   **Target Trait**: Define API endpoints by implementing `target::Target`.
//! *   **Provider Pattern**: Centralize request logic and client configuration with `provider::Provider`.
//! *   **Flexible Authentication**: Use `http::AuthMethod` for Basic, Bearer, or custom closure-based authentication (e.g., `AuthMethod::header_api_key`).
//! *   **Centralized Timeout**: Set a default timeout at the `Provider` level.
//! *   **Middleware Support**: Optional `reqwest-middleware` integration (via `middleware` feature).
//! *   **JSON-RPC Support**: Optional helpers for JSON-RPC 2.0, including batching (via `jsonrpc` feature).
//!
//! # Getting Started
//!
//! 1.  Define an enum/struct for your API target(s).
//! 2.  Implement `target::Target` for your type.
//! 3.  Create a `provider::Provider` instance.
//! 4.  Use the provider to make requests.
//!
//! (See examples directory and specific item documentation for detailed usage.)


pub mod error;
pub use error::Error;
pub mod http;
pub mod provider;
pub mod target;

#[cfg(feature = "jsonrpc")]
pub mod jsonrpc;
