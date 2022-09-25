use std::error::Error;

use chrono::{NaiveDateTime, Utc};
use diesel::{RunQueryDsl, SqliteConnection};
use uuid::Uuid;

use super::schema;

use super::cosigner::Cosigner;
use super::wallet::Wallet;

#[derive(Identifiable, Queryable, Associations)]
#[diesel(belongs_to(Cosigner))]
#[diesel(belongs_to(Wallet))]
#[diesel(table_name = schema::psbt)]
pub struct Psbt {
    pub id: i32,
    pub uuid: String,
    pub data: String,
    pub creation_time: NaiveDateTime,
    pub cosigner_id: i32,
    pub wallet_id: i32,
}

#[derive(Insertable, Associations)]
#[diesel(belongs_to(Cosigner))]
#[diesel(belongs_to(Wallet))]
#[diesel(table_name = schema::psbt)]
pub struct NewPsbt<'a> {
    pub uuid: String,
    pub data: &'a str,
    pub creation_time: NaiveDateTime,
    pub cosigner_id: i32,
    pub wallet_id: i32,
}

impl<'a> NewPsbt<'a> {
    pub fn new(data: &'a str, cosigner_id: i32, wallet_id: i32) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            data,
            creation_time: Utc::now().naive_local(),
            cosigner_id,
            wallet_id,
        }
    }

    pub fn store(&self, connection: &mut SqliteConnection) -> Result<Psbt, Box<dyn Error>> {
        Ok(diesel::insert_into(schema::psbt::table)
            .values(self)
            .get_result(connection)?)
    }
}
