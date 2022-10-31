use diesel::{Connection, SqliteConnection};

#[rustfmt::skip]
mod schema;

mod cosigner;
mod psbt;
mod wallet;

pub use cosigner::{Cosigner, CosignerRecord, CosignerType};
pub use psbt::Psbt;
pub use wallet::{AddressType, Network, Wallet, WalletDescriptors, WalletRecord};

pub fn establish_connection(db_path: &str) -> SqliteConnection {
    SqliteConnection::establish(db_path)
        .unwrap_or_else(|_| panic!("Error connecting to {}", db_path))
}
