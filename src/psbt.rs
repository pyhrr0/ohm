use std::{error::Error, str::FromStr};

use bdk::bitcoin::psbt::PartiallySignedTransaction;
use diesel::SqliteConnection;
use uuid::Uuid;

use crate::db;

pub struct Psbt {
    uuid: Option<String>,
    bdk_handle: PartiallySignedTransaction,
    wallet: Uuid,
}

impl Psbt {
    pub fn new(bdk_handle: PartiallySignedTransaction, wallet: Uuid) -> Self {
        Self {
            uuid: None,
            bdk_handle,
            wallet,
        }
    }

    pub fn from_db(
        connection: &mut SqliteConnection,
        uuid: Option<Uuid>,
    ) -> Result<Option<Self>, Box<dyn Error>> {
        let mut psbts = Self::find(connection, uuid, None)?;

        Ok(match !psbts.is_empty() {
            true => Some(psbts.remove(0)),
            false => None,
        })
    }

    pub fn find(
        connection: &mut SqliteConnection,
        uuid: Option<Uuid>,
        wallet: Option<Uuid>,
    ) -> Result<Vec<Self>, Box<dyn Error>> {
        let records = db::Psbt::find(connection, uuid.as_ref(), wallet.as_ref())?;

        let mut transactions = vec![];
        for record in records {
            transactions.push(Psbt {
                uuid: Some(record.uuid),
                bdk_handle: PartiallySignedTransaction::from_str(&record.base64)?,
                wallet: Uuid::from_str(&record.wallet_uuid)?,
            });
        }

        Ok(transactions)
    }

    pub fn base64(&self) -> String {
        self.bdk_handle.to_string()
    }

    pub fn inner(&mut self) -> &mut PartiallySignedTransaction {
        &mut self.bdk_handle
    }

    pub fn wallet(&self) -> &Uuid {
        &self.wallet
    }

    pub fn uuid(&self) -> Option<&str> {
        self.uuid.as_deref()
    }

    pub fn remove(&mut self, connection: &mut SqliteConnection) -> Result<(), Box<dyn Error>> {
        if let Some(uuid) = &self.uuid {
            db::Psbt::remove(connection, uuid)?;
        }
        self.uuid = None;

        Ok(())
    }

    pub fn save(&mut self, connection: &mut SqliteConnection) -> Result<(), Box<dyn Error>> {
        let base64 = self.base64();
        let mut new_record = db::Psbt::new(&base64, &self.wallet);

        if let Some(uuid) = &self.uuid {
            new_record.uuid = uuid.clone();
        };

        let record = new_record.upsert(connection)?;

        if self.uuid.is_none() {
            self.uuid = Some(record.uuid)
        }

        Ok(())
    }
}
