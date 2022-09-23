use std::error::Error;

use chrono::Local;
use diesel::prelude::*;
use uuid::Uuid;

use super::models::NewPsbt;
use super::schema;

pub fn store_psbt(
    conn: &mut SqliteConnection,
    data: &str,
    cosigner_id: i32,
    wallet_id: i32,
) -> Result<usize, Box<dyn Error>> {
    let num_rows = diesel::insert_into(schema::psbt::table)
        .values(&NewPsbt {
            uuid: &Uuid::new_v4().to_string(),
            data,
            cosigner_id,
            wallet_id,
            creation_time: Local::now().naive_local(),
        })
        .execute(conn)?;

    Ok(num_rows)
}
