use std::error::Error;

use chrono::Local;
use diesel::prelude::*;
use uuid::Uuid;

#[allow(clippy::extra_unused_lifetimes)]
mod models;
mod schema;

pub fn establish_connection(db_path: &str) -> SqliteConnection {
    SqliteConnection::establish(db_path)
        .unwrap_or_else(|_| panic!("Error connecting to {}", db_path))
}

pub fn create_wallet(
    conn: &SqliteConnection,
    _info: &bdk::Wallet<bdk::database::MemoryDatabase>,
    address_type: models::AddressType,
    required_signatures: i32,
) -> Result<usize, Box<dyn Error>> {
    let num_rows = diesel::insert_into(schema::wallet::table)
        .values(&models::NewWallet {
            uuid: &Uuid::new_v4().to_string(),
            address_type,
            receive_descriptor: "TODO",
            receive_address_index: 42,
            change_descriptor: "TODO",
            change_address_index: 42,
            required_signatures,
            creation_time: Local::now().naive_local(),
        })
        .execute(conn)?;

    Ok(num_rows)
}

pub fn register_cosigner(
    conn: &SqliteConnection,
    cosigner_type: models::CosignerType,
    email_address: &str,
    wallet_id: i32,
) -> Result<usize, Box<dyn Error>> {
    let num_rows = diesel::insert_into(schema::cosigner::table)
        .values(&models::NewCosigner {
            uuid: &Uuid::new_v4().to_string(),
            cosigner_type,
            email_address,
            wallet_id,
        })
        .execute(conn)?;

    Ok(num_rows)
}

pub fn create_psbt(
    conn: &SqliteConnection,
    data: &str,
    cosigner_id: i32,
    wallet_id: i32,
) -> Result<usize, Box<dyn Error>> {
    let num_rows = diesel::insert_into(schema::psbt::table)
        .values(&models::NewPsbt {
            uuid: &Uuid::new_v4().to_string(),
            data,
            cosigner_id,
            wallet_id,
        })
        .execute(conn)?;

    Ok(num_rows)
}

pub fn create_xpub(
    conn: &SqliteConnection,
    derivation_path: &str,
    fingerprint: &str,
    data: &str,
    cosigner_id: i32,
    wallet_id: i32,
) -> Result<usize, Box<dyn Error>> {
    let num_rows = diesel::insert_into(schema::xpub::table)
        .values(&models::NewXpub {
            uuid: &Uuid::new_v4().to_string(),
            derivation_path,
            fingerprint,
            data,
            cosigner_id,
            wallet_id,
        })
        .execute(conn)?;

    Ok(num_rows)
}

pub fn create_xprv(
    conn: &SqliteConnection,
    mnemonic: &str,
    fingerprint: &str,
    data: &str,
    cosigner_id: i32,
    wallet_id: i32,
) -> Result<usize, Box<dyn Error>> {
    let num_rows = diesel::insert_into(schema::xprv::table)
        .values(&models::NewXprv {
            uuid: &Uuid::new_v4().to_string(),
            mnemonic,
            fingerprint,
            data,
            cosigner_id,
            wallet_id,
        })
        .execute(conn)?;

    Ok(num_rows)
}
