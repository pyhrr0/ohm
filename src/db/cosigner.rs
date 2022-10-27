use std::error::Error;

use bdk::bitcoin::util::bip32;
use chrono::NaiveDateTime;
use chrono::Utc;
use diesel::deserialize::FromSql;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::SmallInt;
use diesel::sqlite::{Sqlite, SqliteValue};
use diesel::{deserialize, serialize, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use email_address::EmailAddress;
use uuid::Uuid;

use super::schema;
use schema::cosigner::dsl;

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
            1 => Ok(CosignerType::Internal),
            2 => Ok(CosignerType::External),
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
pub struct CosignerRecord {
    pub id: i32,
    pub uuid: String,
    pub type_: CosignerType,
    pub email_address: Option<String>,
    pub xpub: String,
    pub xprv: Option<String>,
    pub creation_time: NaiveDateTime,
    pub wallet_uuid: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = schema::cosigner)]
pub struct Cosigner {
    pub uuid: String,
    pub type_: CosignerType,
    pub email_address: Option<String>,
    pub xpub: String,
    pub xprv: Option<String>,
    pub creation_time: NaiveDateTime,
    pub wallet_uuid: Option<String>,
}

impl Cosigner {
    pub fn new(
        type_: CosignerType,
        email_address: Option<&EmailAddress>,
        xprv: Option<&bip32::ExtendedPrivKey>,
        xpub: &bip32::ExtendedPubKey,
        wallet_uuid: Option<&Uuid>,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            type_,
            email_address: email_address.map(|eml| eml.to_string()),
            xprv: xprv.map(|xprv| xprv.to_string()),
            xpub: xpub.to_string(),
            creation_time: Utc::now().naive_local(),
            wallet_uuid: wallet_uuid.map(|uuid| uuid.to_string()),
        }
    }

    pub fn store(
        &self,
        connection: &mut SqliteConnection,
    ) -> Result<CosignerRecord, Box<dyn Error>> {
        Ok(diesel::insert_into(schema::cosigner::table)
            .values(self)
            .get_result(connection)?)
    }

    pub fn fetch(
        connection: &mut SqliteConnection,
        uuid: Option<Uuid>,
        email_address: Option<EmailAddress>,
        xpub: Option<bip32::ExtendedPubKey>,
    ) -> Result<Vec<CosignerRecord>, Box<dyn Error>> {
        let mut query = dsl::cosigner.into_boxed();

        if let Some(uuid) = uuid {
            query = query.filter(schema::cosigner::uuid.eq(uuid.to_string()));
        }

        if let Some(email_address) = email_address {
            query = query.filter(schema::cosigner::email_address.eq(email_address.to_string()));
        }

        if let Some(xpub) = xpub {
            query = query.filter(schema::cosigner::xpub.eq(xpub.to_string()));
        }

        Ok(query.load::<CosignerRecord>(connection)?)
    }

    pub fn remove(connection: &mut SqliteConnection, uuid: Uuid) -> Result<usize, Box<dyn Error>> {
        Ok(
            diesel::delete(dsl::cosigner.filter(schema::cosigner::uuid.eq(uuid.to_string())))
                .execute(connection)?,
        )
    }
}
