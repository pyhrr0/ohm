use std::error::Error;

use chrono::{NaiveDateTime, Utc};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use uuid::Uuid;

use super::schema;
use schema::psbt::dsl;

#[derive(Identifiable, Queryable)]
#[diesel(table_name = schema::psbt)]
pub struct Psbt {
    pub id: i32,
    pub uuid: String,
    pub base64: String,
    pub creation_time: NaiveDateTime,
    pub wallet_uuid: String,
}

#[derive(Insertable)]
#[diesel(table_name = schema::psbt)]
pub struct NewPsbt<'a> {
    pub uuid: String,
    pub base64: &'a str,
    pub creation_time: NaiveDateTime,
    pub wallet_uuid: &'a str,
}

impl<'a> NewPsbt<'a> {
    pub fn new(base64: &'a str, wallet_id: &'a str) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            base64,
            creation_time: Utc::now().naive_local(),
            wallet_uuid: wallet_id,
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
        wallet_uuid: Option<&str>,
    ) -> Result<Vec<Psbt>, Box<dyn Error>> {
        let mut query = dsl::psbt.into_boxed();

        if let Some(uuid) = uuid {
            query = query.filter(schema::psbt::uuid.eq(uuid));
        }

        if let Some(wallet_uuid) = wallet_uuid {
            query = query.filter(schema::psbt::wallet_uuid.eq(wallet_uuid));
        }

        Ok(query.load::<Psbt>(connection)?)
    }

    pub fn remove(connection: &mut SqliteConnection, uuid: &str) -> Result<usize, Box<dyn Error>> {
        Ok(diesel::delete(dsl::psbt.filter(schema::psbt::uuid.eq(uuid))).execute(connection)?)
    }
}
