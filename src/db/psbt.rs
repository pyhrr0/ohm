use std::error::Error;

use chrono::{NaiveDateTime, Utc};
use diesel::{AsChangeset, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use uuid::Uuid;

use super::{schema, schema::psbt::dsl};

#[derive(Identifiable, Queryable)]
#[diesel(table_name = schema::psbt)]
pub struct PsbtRecord {
    pub id: i32,
    pub uuid: String,
    pub base64: String,
    pub creation_time: NaiveDateTime,
    pub wallet_uuid: String,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = schema::psbt)]
pub struct Psbt<'a> {
    pub uuid: String,
    pub base64: &'a str,
    pub creation_time: NaiveDateTime,
    pub wallet_uuid: String,
}

impl<'a> Psbt<'a> {
    pub fn new(base64: &'a str, wallet_id: &'a Uuid) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            base64,
            creation_time: Utc::now().naive_local(),
            wallet_uuid: wallet_id.to_string(),
        }
    }

    pub fn upsert(&self, connection: &mut SqliteConnection) -> Result<PsbtRecord, Box<dyn Error>> {
        Ok(diesel::insert_into(schema::psbt::table)
            .values(self)
            .on_conflict(dsl::uuid)
            .do_update()
            .set(self)
            .get_result(connection)?)
    }

    pub fn find(
        connection: &mut SqliteConnection,
        uuid: Option<&Uuid>,
        wallet_uuid: Option<&Uuid>,
    ) -> Result<Vec<PsbtRecord>, Box<dyn Error>> {
        let mut query = dsl::psbt.into_boxed();

        if let Some(uuid) = uuid {
            query = query.filter(schema::psbt::uuid.eq(uuid.to_string()));
        }

        if let Some(wallet_uuid) = wallet_uuid {
            query = query.filter(schema::psbt::wallet_uuid.eq(wallet_uuid.to_string()));
        }

        Ok(query.load::<PsbtRecord>(connection)?)
    }

    pub fn remove(connection: &mut SqliteConnection, uuid: &str) -> Result<usize, Box<dyn Error>> {
        Ok(diesel::delete(dsl::psbt.filter(schema::psbt::uuid.eq(uuid))).execute(connection)?)
    }
}
