use std::io::Write;

use chrono::NaiveDateTime;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::SmallInt;
use diesel::sqlite::Sqlite;
use diesel::{deserialize, serialize};

use crate::db::schema::{cosigner, psbt, wallet, xprv, xpub};

#[derive(AsExpression, Clone, Copy, Debug)]
#[sql_type = "SmallInt"]
pub enum CosignerType {
    Internal = 0,
    External = 1,
}

impl ToSql<SmallInt, Sqlite> for CosignerType {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Sqlite>) -> serialize::Result {
        <i16 as ToSql<SmallInt, Sqlite>>::to_sql(&(*self as i16), out)
    }
}

impl FromSql<SmallInt, Sqlite> for CosignerType {
    fn from_sql(bytes: Option<&<Sqlite as Backend>::RawValue>) -> deserialize::Result<Self> {
        match <i16 as FromSql<SmallInt, Sqlite>>::from_sql(bytes)? {
            0 => Ok(CosignerType::Internal),
            1 => Ok(CosignerType::External),
            x => Err(format!("Unrecognized address type {}", x).into()),
        }
    }
}

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Wallet)]
#[table_name = "cosigner"]
pub struct Cosigner {
    pub id: i64,
    pub uuid: String,
    pub cosigner_type: CosignerType,
    pub email_address: String,
    pub public_key: String,
    pub wallet_id: Option<i32>,
}

#[derive(Insertable, Associations)]
#[belongs_to(Wallet)]
#[table_name = "cosigner"]
pub struct NewCosigner<'a> {
    pub uuid: &'a str,
    pub cosigner_type: CosignerType,
    pub email_address: &'a str,
    pub public_key: &'a str,
    pub wallet_id: Option<i32>,
}

#[derive(AsExpression, Clone, Copy, Debug)]
#[sql_type = "SmallInt"]
pub enum AddressType {
    P2sh = 1,
    P2wsh = 2,
    P2shwsh = 3,
    P2tr = 4,
}

impl ToSql<SmallInt, Sqlite> for AddressType {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Sqlite>) -> serialize::Result {
        <i16 as ToSql<SmallInt, Sqlite>>::to_sql(&(*self as i16), out)
    }
}

impl FromSql<SmallInt, Sqlite> for AddressType {
    fn from_sql(bytes: Option<&<Sqlite as Backend>::RawValue>) -> deserialize::Result<Self> {
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
#[table_name = "wallet"]
pub struct Wallet {
    pub id: i64,
    pub uuid: String,
    pub address_type: AddressType,
    pub receive_descriptor: String,
    pub receive_address_index: i64,
    pub receive_address: String,
    pub change_descriptor: String,
    pub change_address_index: i64,
    pub change_address: String,
    pub required_signatures: i64,
    pub creation_time: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "wallet"]
pub struct NewWallet<'a> {
    pub uuid: &'a str,
    pub address_type: AddressType,
    pub receive_descriptor: &'a str,
    pub receive_address_index: i32,
    pub change_descriptor: &'a str,
    pub change_address_index: i32,
    pub required_signatures: i32,
    pub creation_time: NaiveDateTime,
}

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Cosigner)]
#[belongs_to(Wallet)]
#[table_name = "psbt"]
pub struct Psbt {
    pub id: i64,
    pub uuid: String,
    pub data: String,
    pub cosigner_id: i32,
    pub wallet_id: i32,
}

#[derive(Insertable, Associations)]
#[belongs_to(Cosigner)]
#[belongs_to(Wallet)]
#[table_name = "psbt"]
pub struct NewPsbt<'a> {
    pub uuid: &'a str,
    pub data: &'a str,
    pub cosigner_id: i32,
    pub wallet_id: i32,
}

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Cosigner)]
#[belongs_to(Wallet)]
#[table_name = "xpub"]
pub struct Xpub {
    pub id: i64,
    pub uuid: String,
    pub derivation_path: String,
    pub fingerprint: String,
    pub data: String,
    pub cosigner_id: i32,
    pub wallet_id: i32,
}

#[derive(Insertable, Associations)]
#[belongs_to(Cosigner)]
#[belongs_to(Wallet)]
#[table_name = "xpub"]
pub struct NewXpub<'a> {
    pub uuid: &'a str,
    pub derivation_path: &'a str,
    pub fingerprint: &'a str,
    pub data: &'a str,
    pub cosigner_id: i32,
    pub wallet_id: i32,
}

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Cosigner)]
#[belongs_to(Wallet)]
#[table_name = "xprv"]
pub struct Xprv {
    pub id: i32,
    pub uuid: String,
    pub mnemonic: String,
    pub fingerprint: String,
    pub data: String,
    pub cosigner_id: i32,
    pub wallet_id: i32,
}

#[derive(Insertable, Associations)]
#[belongs_to(Cosigner)]
#[belongs_to(Wallet)]
#[table_name = "xprv"]
pub struct NewXprv<'a> {
    pub uuid: &'a str,
    pub mnemonic: &'a str,
    pub fingerprint: &'a str,
    pub data: &'a str,
    pub cosigner_id: i32,
    pub wallet_id: i32,
}
