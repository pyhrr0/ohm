use std::error::Error;
use std::fmt;
use std::str::FromStr;

use bdk::bitcoin;
use chrono::{NaiveDateTime, Utc};
use diesel::deserialize::FromSql;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::{SmallInt, Text};
use diesel::sqlite::{Sqlite, SqliteValue};
use diesel::{deserialize, serialize, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use int_enum::IntEnum;
use rust_decimal::Decimal;
use uuid::Uuid;

use super::schema;
use schema::cosigner::dsl::cosigner;
use schema::psbt::dsl::psbt;
use schema::wallet::dsl::wallet;
use schema::xprv::dsl::xprv;
use schema::xpub::dsl::xpub;

#[repr(i16)]
#[derive(AsExpression, Debug, Copy, Clone, FromSqlRow, IntEnum)]
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

#[repr(i16)]
#[derive(AsExpression, Debug, Copy, Clone, FromSqlRow, IntEnum)]
#[diesel(sql_type = SmallInt)]
pub enum Network {
    Regtest = 1,
    Testnet = 2,
    Signet = 3,
    Mainnet = 4,
}

impl ToSql<SmallInt, Sqlite> for Network {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(*self as i32);
        Ok(IsNull::No)
    }
}

impl FromSql<SmallInt, Sqlite> for Network {
    fn from_sql(bytes: SqliteValue) -> deserialize::Result<Self> {
        match <i16 as FromSql<SmallInt, Sqlite>>::from_sql(bytes)? {
            1 => Ok(Network::Regtest),
            2 => Ok(Network::Testnet),
            3 => Ok(Network::Signet),
            4 => Ok(Network::Mainnet),
            x => Err(format!("Unrecognized address type {}", x).into()),
        }
    }
}

impl From<bitcoin::Network> for Network {
    fn from(network: bitcoin::Network) -> Self {
        match network {
            bitcoin::Network::Regtest => Self::Regtest,
            bitcoin::Network::Testnet => Self::Testnet,
            bitcoin::Network::Bitcoin => Self::Mainnet,
            bitcoin::Network::Signet => Self::Mainnet,
        }
    }
}

#[derive(AsExpression, Debug, FromSqlRow)]
#[diesel(sql_type = Text)]
pub struct DecimalWrapper(Decimal);

impl fmt::Display for DecimalWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ToSql<Text, Sqlite> for DecimalWrapper {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.to_string());
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Sqlite> for DecimalWrapper {
    fn from_sql(bytes: SqliteValue) -> deserialize::Result<Self> {
        let decimal = Decimal::from_str(&<String as FromSql<Text, Sqlite>>::from_sql(bytes)?)?;
        Ok(DecimalWrapper(decimal))
    }
}

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = schema::wallet)]
pub struct Wallet {
    pub id: i32,
    pub uuid: String,
    pub address_type: AddressType,
    pub network: Network,
    pub receive_descriptor: String,
    pub receive_address_index: i64,
    pub receive_address: String,
    pub change_descriptor: String,
    pub change_address_index: i64,
    pub change_address: String,
    pub balance: DecimalWrapper,
    pub required_signatures: i16,
    pub creation_time: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = schema::wallet)]
pub struct NewWallet<'a> {
    pub uuid: String,
    pub address_type: AddressType,
    pub network: Network,
    pub receive_descriptor: &'a str,
    pub receive_address_index: i64,
    pub change_descriptor: &'a str,
    pub change_address_index: i64,
    pub required_signatures: i16,
    pub balance: DecimalWrapper,
    pub creation_time: NaiveDateTime,
}

impl<'a> NewWallet<'a> {
    pub fn new(
        address_type: AddressType,
        network: Network,
        receive_descriptor: &'a str,
        change_descriptor: &'a str,
        required_signatures: i16,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            address_type,
            network,
            receive_descriptor,
            receive_address_index: 0,
            change_descriptor,
            change_address_index: 0,
            required_signatures,
            balance: DecimalWrapper(Decimal::new(0, 0)),
            creation_time: Utc::now().naive_local(),
        }
    }

    pub fn store(&self, connection: &mut SqliteConnection) -> Result<Wallet, Box<dyn Error>> {
        Ok(diesel::insert_into(schema::wallet::table)
            .values(self)
            .get_result(connection)?)
    }

    pub fn fetch(
        connection: &mut SqliteConnection,
        id: Option<i32>,
        uuid: Option<&str>,
        address_type: Option<AddressType>,
        network: Option<Network>,
    ) -> Result<Vec<Wallet>, Box<dyn Error>> {
        let mut query = wallet.into_boxed();

        if let Some(id) = id {
            query = query.filter(schema::wallet::id.eq(id));
        }

        if let Some(uuid) = uuid {
            query = query.filter(schema::wallet::uuid.eq(uuid));
        }

        if let Some(address_type) = address_type {
            query = query.filter(schema::wallet::address_type.eq(address_type));
        }

        if let Some(network) = network {
            query = query.filter(schema::wallet::network.eq(network));
        }

        Ok(query.load::<Wallet>(connection)?)
    }

    pub fn remove(connection: &mut SqliteConnection, uuid: &str) -> Result<usize, Box<dyn Error>> {
        let wallet_id = wallet
            .select(schema::wallet::id)
            .filter(schema::wallet::uuid.eq(uuid))
            .first::<i32>(connection)?;

        diesel::delete(cosigner.filter(schema::cosigner::wallet_uuid.eq(uuid)))
            .execute(connection)?;
        diesel::delete(xpub.filter(schema::xpub::wallet_id.eq(wallet_id))).execute(connection)?;
        diesel::delete(xprv.filter(schema::xprv::wallet_id.eq(wallet_id))).execute(connection)?;
        diesel::delete(psbt.filter(schema::psbt::wallet_id.eq(wallet_id))).execute(connection)?;

        Ok(diesel::delete(wallet.filter(schema::wallet::uuid.eq(uuid))).execute(connection)?)
    }
}
