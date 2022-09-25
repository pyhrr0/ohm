use std::error::Error;

use chrono::{NaiveDateTime, Utc};
use diesel::deserialize::FromSql;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::SmallInt;
use diesel::sqlite::{Sqlite, SqliteValue};
use diesel::{deserialize, serialize};
use diesel::{RunQueryDsl, SqliteConnection};
use uuid::Uuid;

use super::schema;

#[repr(i16)]
#[derive(AsExpression, Debug, Copy, Clone, FromSqlRow)]
#[diesel(sql_type = SmallInt)]
pub enum AddressType {
    P2sh = 1,
    P2wsh = 2,
    P2shwsh = 3,
    P2tr = 4,
}

impl ToSql<SmallInt, Sqlite> for AddressType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(*self as i32);
        Ok(IsNull::No)
    }
}

impl FromSql<SmallInt, Sqlite> for AddressType {
    fn from_sql(bytes: SqliteValue) -> deserialize::Result<Self> {
        match <i16 as FromSql<SmallInt, Sqlite>>::from_sql(bytes)? {
            1 => Ok(AddressType::P2sh),
            2 => Ok(AddressType::P2wsh),
            3 => Ok(AddressType::P2shwsh),
            4 => Ok(AddressType::P2tr),
            x => Err(format!("Unrecognized address type {}", x).into()),
        }
    }
}

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = schema::wallet)]
pub struct Wallet {
    pub id: i32,
    pub uuid: String,
    pub address_type: AddressType,
    pub receive_descriptor: String,
    pub receive_address_index: i64,
    pub receive_address: String,
    pub change_descriptor: String,
    pub change_address_index: i64,
    pub change_address: String,
    pub required_signatures: i16,
    pub creation_time: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = schema::wallet)]
pub struct NewWallet<'a> {
    pub uuid: String,
    pub address_type: AddressType,
    pub receive_descriptor: &'a str,
    pub receive_address_index: i64,
    pub change_descriptor: &'a str,
    pub change_address_index: i64,
    pub required_signatures: i16,
    pub creation_time: NaiveDateTime,
}

impl<'a> NewWallet<'a> {
    pub fn new(
        address_type: AddressType,
        receive_descriptor: &'a str,
        receive_address_index: i64,
        change_descriptor: &'a str,
        change_address_index: i64,
        required_signatures: i16,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            address_type,
            receive_descriptor,
            receive_address_index,
            change_descriptor,
            change_address_index,
            required_signatures,
            creation_time: Utc::now().naive_local(),
        }
    }

    pub fn store(&self, connection: &mut SqliteConnection) -> Result<Wallet, Box<dyn Error>> {
        Ok(diesel::insert_into(schema::wallet::table)
            .values(self)
            .get_result(connection)?)
    }
}
