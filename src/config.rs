use std::net::Ipv4Addr;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub bind_addr: Ipv4Addr,
    pub port: u16,
    pub backend_url: Url,
    pub db_path: PathBuf,
}
