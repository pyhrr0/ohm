use std::{error::Error, fmt, str::FromStr};

use bdk::{bitcoin, Balance};
use chrono::{NaiveDateTime, Utc};
use diesel::{
    deserialize, serialize, sql_types, sqlite, AsChangeset, ExpressionMethods, QueryDsl,
    RunQueryDsl, SqliteConnection,
};
use int_enum::IntEnum;
use rust_decimal::Decimal;
use uuid::Uuid;

use super::{
    schema,
    schema::{cosigner::dsl::cosigner, psbt::dsl::psbt, wallet::dsl},
};

#[repr(i16)]
#[derive(AsExpression, Debug, Copy, Clone, FromSqlRow, IntEnum)]
#[diesel(sql_type = sql_types::SmallInt)]
pub enum AddressType {
    P2sh = 1,
    P2wsh = 2,
    P2shwsh = 3,
}

impl serialize::ToSql<sql_types::SmallInt, sqlite::Sqlite> for AddressType {
    fn to_sql<'b>(
        &'b self,
        out: &mut serialize::Output<'b, '_, sqlite::Sqlite>,
    ) -> serialize::Result {
        out.set_value(*self as i32);
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<sql_types::SmallInt, sqlite::Sqlite> for AddressType {
    fn from_sql(bytes: sqlite::SqliteValue) -> deserialize::Result<Self> {
        match <i16 as deserialize::FromSql<sql_types::SmallInt, sqlite::Sqlite>>::from_sql(bytes)? {
            1 => Ok(AddressType::P2sh),
            2 => Ok(AddressType::P2wsh),
            3 => Ok(AddressType::P2shwsh),
            x => Err(format!("Unrecognized address type {}", x).into()),
        }
    }
}

#[repr(i16)]
#[derive(AsExpression, Debug, Copy, Clone, FromSqlRow, IntEnum)]
#[diesel(sql_type = sql_types::SmallInt)]
pub enum Network {
    Regtest = 1,
    Testnet = 2,
    Mainnet = 3,
}

impl serialize::ToSql<sql_types::SmallInt, sqlite::Sqlite> for Network {
    fn to_sql<'b>(
        &'b self,
        out: &mut serialize::Output<'b, '_, sqlite::Sqlite>,
    ) -> serialize::Result {
        out.set_value(*self as i32);
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<sql_types::SmallInt, sqlite::Sqlite> for Network {
    fn from_sql(bytes: sqlite::SqliteValue) -> deserialize::Result<Self> {
        match <i16 as deserialize::FromSql<sql_types::SmallInt, sqlite::Sqlite>>::from_sql(bytes)? {
            1 => Ok(Network::Regtest),
            2 => Ok(Network::Testnet),
            3 => Ok(Network::Mainnet),
            x => Err(format!("Unrecognized address type {}", x).into()),
        }
    }
}

impl From<Network> for bitcoin::Network {
    fn from(network: Network) -> Self {
        match network {
            Network::Regtest => Self::Regtest,
            Network::Testnet => Self::Testnet,
            Network::Mainnet => Self::Bitcoin,
        }
    }
}

#[derive(AsExpression, Debug, FromSqlRow)]
#[diesel(sql_type = sql_types::Text)]
pub struct DecimalWrapper(pub Decimal);

impl fmt::Display for DecimalWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl serialize::ToSql<sql_types::Text, sqlite::Sqlite> for DecimalWrapper {
    fn to_sql<'b>(
        &'b self,
        out: &mut serialize::Output<'b, '_, sqlite::Sqlite>,
    ) -> serialize::Result {
        out.set_value(self.to_string());
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<sql_types::Text, sqlite::Sqlite> for DecimalWrapper {
    fn from_sql(bytes: sqlite::SqliteValue) -> deserialize::Result<Self> {
        let decimal = Decimal::from_str(&<String as deserialize::FromSql<
            sql_types::Text,
            sqlite::Sqlite,
        >>::from_sql(bytes)?)?;
        Ok(DecimalWrapper(decimal))
    }
}

pub struct WalletDescriptors {
    pub receive_descriptor: String,
    pub receive_descriptor_watch_only: String,
    pub change_descriptor: String,
    pub change_descriptor_watch_only: String,
}

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = schema::wallet)]
pub struct WalletRecord {
    pub id: i32,
    pub uuid: String,
    pub address_type: AddressType,
    pub network: Network,
    pub receive_descriptor: String,
    pub receive_descriptor_watch_only: String,
    pub receive_address_index: i64,
    pub receive_address: String,
    pub change_descriptor: String,
    pub change_descriptor_watch_only: String,
    pub change_address_index: i64,
    pub change_address: String,
    pub required_signatures: i16,
    pub balance: DecimalWrapper,
    pub creation_time: NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = schema::wallet)]
pub struct Wallet<'a> {
    pub uuid: String,
    pub address_type: AddressType,
    pub network: Network,
    pub receive_descriptor: &'a str,
    pub receive_descriptor_watch_only: &'a str,
    pub receive_address_index: i64,
    pub change_descriptor: &'a str,
    pub change_descriptor_watch_only: &'a str,
    pub change_address_index: i64,
    pub required_signatures: i16,
    pub balance: DecimalWrapper,
    pub creation_time: NaiveDateTime,
}

impl<'a> Wallet<'a> {
    pub fn new(
        address_type: AddressType,
        network: Network,
        required_signatures: i16,
        balance: &Balance,
        descriptors: &'a WalletDescriptors,
        receive_address_index: i64,
        change_address_index: i64,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            address_type,
            network,
            receive_descriptor: &descriptors.receive_descriptor,
            receive_descriptor_watch_only: &descriptors.receive_descriptor_watch_only,
            receive_address_index,
            change_descriptor: &descriptors.change_descriptor,
            change_descriptor_watch_only: &descriptors.change_descriptor_watch_only,
            change_address_index,
            required_signatures,
            balance: DecimalWrapper(Decimal::from(balance.confirmed)),
            creation_time: Utc::now().naive_local(),
        }
    }

    pub fn upsert(
        &self,
        connection: &mut SqliteConnection,
    ) -> Result<WalletRecord, Box<dyn Error>> {
        Ok(diesel::insert_into(schema::wallet::table)
            .values(self)
            .on_conflict(dsl::uuid)
            .do_update()
            .set(self)
            .get_result(connection)?)
    }

    pub fn find(
        connection: &mut SqliteConnection,
        uuid: Option<&Uuid>,
        address_type: Option<AddressType>,
        network: Option<Network>,
        receive_descriptor: Option<&str>,
    ) -> Result<Vec<WalletRecord>, Box<dyn Error>> {
        let mut query = dsl::wallet.into_boxed();

        if let Some(uuid) = uuid {
            query = query.filter(schema::wallet::uuid.eq(uuid.to_string()));
        }

        if let Some(address_type) = address_type {
            query = query.filter(schema::wallet::address_type.eq(address_type));
        }

        if let Some(network) = network {
            query = query.filter(schema::wallet::network.eq(network));
        }

        if let Some(descriptor) = receive_descriptor {
            query = query.filter(schema::wallet::receive_descriptor_watch_only.eq(descriptor));
        }

        Ok(query.load::<WalletRecord>(connection)?)
    }

    pub fn remove(connection: &mut SqliteConnection, uuid: &str) -> Result<usize, Box<dyn Error>> {
        diesel::delete(cosigner.filter(schema::cosigner::wallet_uuid.eq(uuid.to_string())))
            .execute(connection)?;
        diesel::delete(psbt.filter(schema::psbt::wallet_uuid.eq(uuid.to_string())))
            .execute(connection)?;

        Ok(
            diesel::delete(dsl::wallet.filter(schema::wallet::uuid.eq(uuid.to_string())))
                .execute(connection)?,
        )
    }
}
