use std::error::Error;

use chrono::Local;
use diesel::prelude::*;
use uuid::Uuid;

use super::models::NewXprv;
use super::schema;

pub fn store_xprv(
    conn: &mut SqliteConnection,
    mnemonic: &str,
    fingerprint: &str,
    data: &str,
    cosigner_id: i32,
    wallet_id: i32,
) -> Result<usize, Box<dyn Error>> {
    let num_rows = diesel::insert_into(schema::xprv::table)
        .values(&NewXprv {
            uuid: &Uuid::new_v4().to_string(),
            mnemonic,
            fingerprint,
            data,
            cosigner_id,
            wallet_id,
            creation_time: Local::now().naive_local(),
        })
        .execute(conn)?;

    Ok(num_rows)
}
