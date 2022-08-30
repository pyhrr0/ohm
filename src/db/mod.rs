use std::error::Error;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

mod models;
mod schema;

pub fn establish_connection(db_path: &str) -> SqliteConnection {
    SqliteConnection::establish(db_path)
        .unwrap_or_else(|_| panic!("Error connecting to {}", db_path))
}

pub fn create_wallet(
    conn: &SqliteConnection,
    address_type: models::AddressType,
    recv_descriptor: &str,
    recv_address_index: Option<i32>,
    chng_descriptor: &str,
    chng_address_index: Option<i32>,
    required_signatures: i32,
    creation_time: NaiveDateTime,
) -> Result<usize, Box<dyn Error>> {
    let num_rows = diesel::insert_into(schema::wallet::table)
        .values(&models::NewWallet {
            uuid: &Uuid::new_v4().to_string(),
            address_type: address_type,
            receive_descriptor: recv_descriptor,
            receive_address_index: recv_address_index.unwrap_or(0),
            change_descriptor: chng_descriptor,
            change_address_index: chng_address_index.unwrap_or(0),
            required_signatures: required_signatures,
            creation_time: creation_time,
        })
        .execute(conn)?;

    Ok(num_rows)
}

pub fn register_cosigner(
    conn: &SqliteConnection,
    cosigner_type: models::CosignerType,
    email: &str,
    wallet_id: i32,
) -> Result<usize, Box<dyn Error>> {
    let num_rows = diesel::insert_into(schema::cosigner::table)
        .values(&models::NewCosigner {
            uuid: &Uuid::new_v4().to_string(),
            cosigner_type: cosigner_type,
            email: email,
            wallet_id: wallet_id,
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
            data: data,
            cosigner_id: cosigner_id,
            wallet_id: wallet_id,
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
            derivation_path: derivation_path,
            fingerprint: fingerprint,
            data: data,
            cosigner_id: cosigner_id,
            wallet_id: wallet_id,
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
            mnemonic: mnemonic,
            fingerprint: fingerprint,
            data: data,
            cosigner_id: cosigner_id,
            wallet_id: wallet_id,
        })
        .execute(conn)?;

    Ok(num_rows)
}
