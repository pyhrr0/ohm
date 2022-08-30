#[macro_use]
extern crate diesel;

pub mod db;
pub mod grpc;

mod config;
pub use config::Config;
