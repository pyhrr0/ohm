use std::error::Error;

use chrono::NaiveDateTime;
use chrono::Utc;
use diesel::deserialize::FromSql;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::SmallInt;
use diesel::sqlite::{Sqlite, SqliteValue};
use diesel::{deserialize, serialize, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use uuid::Uuid;

use super::schema;
use schema::cosigner::dsl::cosigner;

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
    pub xpub: String,
    pub xprv: Option<String>,
    pub creation_time: NaiveDateTime,
    pub wallet_uuid: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = schema::cosigner)]
pub struct NewCosigner<'a> {
    pub uuid: String,
    pub type_: CosignerType,
    pub email_address: &'a str,
    pub xpub: &'a str,
    pub xprv: Option<&'a str>,
    pub creation_time: NaiveDateTime,
    pub wallet_uuid: Option<&'a str>,
}

impl<'a> NewCosigner<'a> {
    pub fn new(
        type_: CosignerType,
        email_address: &'a str,
        xprv: Option<&'a str>,
        xpub: &'a str,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            type_,
            email_address,
            xpub,
            xprv,
            creation_time: Utc::now().naive_local(),
            wallet_uuid: None,
        }
    }

    pub fn store(&self, connection: &mut SqliteConnection) -> Result<Cosigner, Box<dyn Error>> {
        Ok(diesel::insert_into(schema::cosigner::table)
            .values(self)
            .get_result(connection)?)
    }

    pub fn fetch(
        connection: &mut SqliteConnection,
        uuid: Option<&str>,
        email_address: Option<&str>,
        xpub: Option<&str>,
    ) -> Result<Vec<Cosigner>, Box<dyn Error>> {
        let mut query = cosigner.into_boxed();

        if let Some(uuid) = uuid {
            query = query.filter(schema::cosigner::uuid.eq(uuid));
        }

        if let Some(email_address) = email_address {
            query = query.filter(schema::cosigner::email_address.eq(email_address));
        }

        if let Some(xpub) = xpub {
            query = query.filter(schema::cosigner::xpub.eq(xpub));
        }

        Ok(query.load::<Cosigner>(connection)?)
    }

    pub fn remove(connection: &mut SqliteConnection, uuid: &str) -> Result<usize, Box<dyn Error>> {
        Ok(diesel::delete(cosigner.filter(schema::cosigner::uuid.eq(uuid))).execute(connection)?)
    }
}
