use std::error::Error;

use chrono::{NaiveDateTime, Utc};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use uuid::Uuid;

use super::schema;
use schema::xprv::dsl::xprv;

use super::cosigner::Cosigner;
use super::wallet::Wallet;

#[derive(Identifiable, Queryable, Associations)]
#[diesel(belongs_to(Cosigner))]
#[diesel(belongs_to(Wallet))]
#[diesel(table_name = schema::xprv)]
pub struct Xprv {
    pub id: i32,
    pub uuid: String,
    pub mnemonic: String,
    pub fingerprint: String,
    pub data: String,
    pub creation_time: NaiveDateTime,
    pub cosigner_id: i32,
    pub wallet_id: i32,
}

#[derive(Insertable, Associations)]
#[diesel(belongs_to(Cosigner))]
#[diesel(belongs_to(Wallet))]
#[diesel(table_name = schema::xprv)]
pub struct NewXprv<'a> {
    pub uuid: String,
    pub mnemonic: &'a str,
    pub fingerprint: &'a str,
    pub data: &'a str,
    pub creation_time: NaiveDateTime,
    pub cosigner_id: i32,
    pub wallet_id: i32,
}

impl<'a> NewXprv<'a> {
    pub fn new(
        mnemonic: &'a str,
        fingerprint: &'a str,
        data: &'a str,
        cosigner_id: i32,
        wallet_id: i32,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            mnemonic,
            fingerprint,
            data,
            creation_time: Utc::now().naive_local(),
            cosigner_id,
            wallet_id,
        }
    }

    pub fn store(&self, connection: &mut SqliteConnection) -> Result<Xprv, Box<dyn Error>> {
        Ok(diesel::insert_into(schema::xprv::table)
            .values(self)
            .get_result(connection)?)
    }

    pub fn fetch(
        connection: &mut SqliteConnection,
        uuid: &str,
    ) -> Result<Vec<Xprv>, Box<dyn Error>> {
        Ok(xprv
            .filter(schema::xprv::uuid.eq(uuid))
            .load::<Xprv>(connection)?)
    }

    pub fn remove(connection: &mut SqliteConnection, uuid: &str) -> Result<usize, Box<dyn Error>> {
        Ok(diesel::delete(xprv.filter(schema::xprv::uuid.eq(uuid))).execute(connection)?)
    }
}
