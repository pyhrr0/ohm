mod grpc;
mod config;

pub use config::Config;
pub use grpc::{create_server, create_client};
