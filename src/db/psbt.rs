use std::error::Error;

use chrono::{NaiveDateTime, Utc};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use uuid::Uuid;

use super::schema;
use schema::psbt::dsl::psbt;

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

    pub fn fetch(
        connection: &mut SqliteConnection,
        uuid: Option<&str>,
        wallet_id: Option<i32>,
    ) -> Result<Vec<Psbt>, Box<dyn Error>> {
        let mut query = psbt.into_boxed();

        if let Some(uuid) = uuid {
            query = query.filter(schema::psbt::uuid.eq(uuid));
        }

        if let Some(wallet_id) = wallet_id {
            query = query.filter(schema::psbt::wallet_id.eq(wallet_id));
        }

        Ok(query.load::<Psbt>(connection)?)
    }

    pub fn remove(connection: &mut SqliteConnection, uuid: &str) -> Result<usize, Box<dyn Error>> {
        Ok(diesel::delete(psbt.filter(schema::psbt::uuid.eq(uuid))).execute(connection)?)
    }
}
