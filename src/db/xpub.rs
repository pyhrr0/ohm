use std::error::Error;

use chrono::Local;
use diesel::prelude::*;
use uuid::Uuid;

use super::models::NewXpub;
use super::schema;

pub fn store_xpub(
    conn: &mut SqliteConnection,
    derivation_path: &str,
    fingerprint: &str,
    data: &str,
    cosigner_id: i32,
    wallet_id: i32,
) -> Result<usize, Box<dyn Error>> {
    let num_rows = diesel::insert_into(schema::xpub::table)
        .values(&NewXpub {
            uuid: &Uuid::new_v4().to_string(),
            derivation_path,
            fingerprint,
            data,
            cosigner_id,
            wallet_id,
            creation_time: Local::now().naive_local(),
        })
        .execute(conn)?;

    Ok(num_rows)
}
