use std::error::Error;

use chrono::NaiveDateTime;
use chrono::Utc;
use diesel::deserialize::FromSql;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::SmallInt;
use diesel::sqlite::{Sqlite, SqliteValue};
use diesel::{deserialize, serialize};
use diesel::{RunQueryDsl, SqliteConnection};
use uuid::Uuid;

use super::schema;

#[repr(i16)]
#[derive(AsExpression, Debug, Clone, Copy, FromSqlRow)]
#[diesel(sql_type = SmallInt)]
pub enum CosignerType {
    Internal = 1,
    External = 2,
}

impl FromSql<SmallInt, Sqlite> for CosignerType {
    fn from_sql(value: SqliteValue) -> deserialize::Result<Self> {
        match <i16 as FromSql<SmallInt, Sqlite>>::from_sql(value)? {
            0 => Ok(CosignerType::Internal),
            1 => Ok(CosignerType::External),
            x => Err(format!("Unrecognized address type {}", x).into()),
        }
    }
}

impl ToSql<SmallInt, Sqlite> for CosignerType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(*self as i32);
        Ok(IsNull::No)
    }
}

#[derive(Identifiable, Queryable)]
#[diesel(table_name = schema::cosigner)]
pub struct Cosigner {
    pub id: i32,
    pub uuid: String,
    pub type_: CosignerType,
    pub email_address: String,
    pub public_key: String,
    pub creation_time: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = schema::cosigner)]
pub struct NewCosigner<'a> {
    pub uuid: String,
    pub type_: CosignerType,
    pub email_address: &'a str,
    pub public_key: &'a str,
    pub creation_time: NaiveDateTime,
}

impl<'a> NewCosigner<'a> {
    pub fn new(email_address: &'a str, public_key: &'a str, type_: CosignerType) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            type_,
            email_address,
            public_key,
            creation_time: Utc::now().naive_local(),
        }
    }

    pub fn store(&self, connection: &mut SqliteConnection) -> Result<Cosigner, Box<dyn Error>> {
        Ok(diesel::insert_into(schema::cosigner::table)
            .values(self)
            .get_result(connection)?)
    }
}
