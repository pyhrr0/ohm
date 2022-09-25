#[allow(clippy::derive_partial_eq_without_eq)]
pub mod proto;

mod service;
pub use service::{Client, OhmResponse as Response, Servicer as Server};
