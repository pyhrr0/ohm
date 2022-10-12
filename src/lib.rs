#[macro_use]
extern crate diesel;

mod db;

mod grpc;
pub use grpc::{proto, Client, Response, Server};

mod wallet;
pub use wallet::{Cosigner, CosignerType};

mod config;
pub use config::Config;
