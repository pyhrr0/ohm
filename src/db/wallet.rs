use std::error::Error;

use chrono::Local;
use diesel::prelude::*;
use uuid::Uuid;

use super::models::{AddressType, NewWallet};
use super::schema;

pub fn store_wallet(
    conn: &mut SqliteConnection,
    _info: &bdk::Wallet<bdk::database::MemoryDatabase>,
    address_type: AddressType,
    required_signatures: i16,
) -> Result<usize, Box<dyn Error>> {
    let num_rows = diesel::insert_into(schema::wallet::table)
        .values(&NewWallet {
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
