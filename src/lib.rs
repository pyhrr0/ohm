#[macro_use]
extern crate diesel;

mod db;

mod config;
mod grpc;

pub use config::Config;
pub use grpc::{create_client, create_server};
