use diesel::{Connection, SqliteConnection};

mod schema;

mod cosigner;
mod psbt;
mod wallet;

pub use cosigner::{Cosigner, CosignerRecord, CosignerType};
pub use psbt::NewPsbt as Psbt;
pub use wallet::{AddressType, Network, Wallet, WalletDescriptors, WalletRecord};

pub fn establish_connection(db_path: &str) -> SqliteConnection {
    SqliteConnection::establish(db_path)
        .unwrap_or_else(|_| panic!("Error connecting to {}", db_path))
}
