use diesel::prelude::*;

mod cosigner;
pub mod models;
mod psbt;
pub mod schema;
mod wallet;
mod xprv;
mod xpub;

pub use cosigner::store_cosigner;
pub use psbt::store_psbt;
pub use wallet::store_wallet;
pub use xprv::store_xprv;
pub use xpub::store_xpub;

pub fn establish_connection(db_path: &str) -> SqliteConnection {
    SqliteConnection::establish(db_path)
        .unwrap_or_else(|_| panic!("Error connecting to {}", db_path))
}
