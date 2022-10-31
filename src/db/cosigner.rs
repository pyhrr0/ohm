use std::error::Error;

use bdk::bitcoin::util::bip32;
use chrono::{NaiveDateTime, Utc};
use diesel::{
    deserialize, serialize, sql_types, sqlite, AsChangeset, ExpressionMethods, QueryDsl,
    RunQueryDsl, SqliteConnection,
};
use email_address::EmailAddress;
use uuid::Uuid;

use super::{schema, schema::cosigner::dsl};

#[repr(i16)]
#[derive(AsExpression, Debug, Clone, Copy, FromSqlRow)]
#[diesel(sql_type = sql_types::SmallInt)]
pub enum CosignerType {
    Internal = 1,
    External = 2,
}

impl serialize::ToSql<sql_types::SmallInt, sqlite::Sqlite> for CosignerType {
    fn to_sql<'b>(
        &'b self,
        out: &mut serialize::Output<'b, '_, sqlite::Sqlite>,
    ) -> serialize::Result {
        out.set_value(*self as i32);
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<sql_types::SmallInt, sqlite::Sqlite> for CosignerType {
    fn from_sql(value: sqlite::SqliteValue) -> deserialize::Result<Self> {
        match <i16 as deserialize::FromSql<sql_types::SmallInt, sqlite::Sqlite>>::from_sql(value)? {
            1 => Ok(CosignerType::Internal),
            2 => Ok(CosignerType::External),
            x => Err(format!("Unrecognized cosigner type {}", x).into()),
        }
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

#[derive(Insertable, AsChangeset)]
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
            email_address: email_address.map(|email| email.to_string()),
            xprv: xprv.map(|xprv| xprv.to_string()),
            xpub: xpub.to_string(),
            creation_time: Utc::now().naive_local(),
            wallet_uuid: wallet_uuid.map(|uuid| uuid.to_string()),
        }
    }

    pub fn upsert(
        &self,
        connection: &mut SqliteConnection,
    ) -> Result<CosignerRecord, Box<dyn Error>> {
        Ok(diesel::insert_into(schema::cosigner::table)
            .values(self)
            .on_conflict(dsl::uuid)
            .do_update()
            .set(self)
            .get_result(connection)?)
    }

    pub fn find(
        connection: &mut SqliteConnection,
        uuid: Option<&Uuid>,
        email_address: Option<&EmailAddress>,
        xpub: Option<&bip32::ExtendedPubKey>,
        wallet_uuid: Option<&Uuid>,
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

        if let Some(uuid) = wallet_uuid {
            query = query.filter(schema::cosigner::wallet_uuid.eq(uuid.to_string()));
        }

        Ok(query.load::<CosignerRecord>(connection)?)
    }

    pub fn remove(connection: &mut SqliteConnection, uuid: &str) -> Result<usize, Box<dyn Error>> {
        Ok(
            diesel::delete(dsl::cosigner.filter(schema::cosigner::uuid.eq(uuid.to_string())))
                .execute(connection)?,
        )
    }
}
