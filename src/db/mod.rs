use diesel::{Connection, SqliteConnection};

mod schema;

mod cosigner;
mod psbt;
mod wallet;
mod xprv;
mod xpub;

pub use cosigner::{CosignerType, NewCosigner as Cosigner};
pub use psbt::NewPsbt as Psbt;
pub use wallet::{AddressType, Network, NewWallet as Wallet};
pub use xprv::NewXprv as Xprv;
pub use xpub::NewXpub as Xpub;

pub fn establish_connection(db_path: &str) -> SqliteConnection {
    SqliteConnection::establish(db_path)
        .unwrap_or_else(|_| panic!("Error connecting to {}", db_path))
}
