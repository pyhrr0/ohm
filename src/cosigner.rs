use std::{error::Error, str::FromStr};

use bdk::{
    bitcoin::{secp256k1, util::bip32},
    keys::{bip39, DerivableKey, ExtendedKey},
};
use diesel::SqliteConnection;
use email_address::EmailAddress;
use uuid::Uuid;

use super::Network;
use crate::db;
pub use db::CosignerType;

pub struct Cosigner {
    uuid: Option<String>,
    type_: CosignerType,
    email_address: Option<EmailAddress>,
    xpub: bip32::ExtendedPubKey,
    xprv: Option<bip32::ExtendedPrivKey>,
    wallet: Option<Uuid>,
}

impl Cosigner {
    pub fn new(
        type_: CosignerType,
        email_address: Option<EmailAddress>,
        xpub: Option<bip32::ExtendedPubKey>,
        network: Option<Network>,
    ) -> Result<Self, Box<dyn Error>> {
        let (xprv, xpub) = match type_ {
            CosignerType::Internal => {
                if let Some(network) = network {
                    let (xprv, xpub) =
                        Self::generate_key_pair(network).map_err(|err| -> Box<dyn Error> {
                            format!("failed to create a key pair: {}", err).into()
                        })?;
                    Ok((Some(xprv), xpub))
                } else {
                    Err("CosignerType::Internal requires a network to be supplied")
                }
            }
            CosignerType::External => {
                if let Some(xpub) = xpub {
                    Ok((None, xpub))
                } else {
                    Err("CosignerType::External requires a xpub to be supplied")
                }
            }
        }?;

        Ok(Self {
            uuid: None,
            type_,
            email_address,
            xprv,
            xpub,
            wallet: None,
        })
    }

    pub fn from_db(
        connection: &mut SqliteConnection,
        uuid: Option<Uuid>,
    ) -> Result<Option<Self>, Box<dyn Error>> {
        let mut cosigners = Self::find(connection, uuid, None, None, None)?;

        Ok(match !cosigners.is_empty() {
            true => Some(cosigners.remove(0)),
            false => None,
        })
    }

    pub fn find(
        connection: &mut SqliteConnection,
        uuid: Option<Uuid>,
        email_address: Option<EmailAddress>,
        xpub: Option<bip32::ExtendedPubKey>,
        wallet: Option<Uuid>,
    ) -> Result<Vec<Self>, Box<dyn Error>> {
        let records = db::Cosigner::find(
            connection,
            uuid.as_ref(),
            email_address.as_ref(),
            xpub.as_ref(),
            wallet.as_ref(),
        )?;

        let mut cosigners = vec![];
        for record in records {
            cosigners.push(Cosigner {
                uuid: Some(record.uuid),
                type_: record.type_,
                email_address: record
                    .email_address
                    .map(|email| EmailAddress::from_str(&email))
                    .transpose()?,
                xprv: record
                    .xprv
                    .map(|xprv| bip32::ExtendedPrivKey::from_str(&xprv))
                    .transpose()?,
                xpub: bip32::ExtendedPubKey::from_str(&record.xpub)?,
                wallet: record
                    .wallet_uuid
                    .map(|uuid| Uuid::from_str(&uuid))
                    .transpose()?,
            });
        }

        Ok(cosigners)
    }

    fn generate_key_pair(
        network: Network,
    ) -> Result<(bip32::ExtendedPrivKey, bip32::ExtendedPubKey), Box<dyn Error>> {
        let mnemonic = bip39::Mnemonic::generate_in_with(
            &mut secp256k1::rand::thread_rng(),
            bip39::Language::English,
            24,
        )?;

        let xkey: ExtendedKey = mnemonic.clone().into_extended_key()?;
        let xprv: bip32::ExtendedPrivKey = xkey.into_xprv(network.into()).unwrap();

        let xkey: ExtendedKey = mnemonic.into_extended_key()?;
        let secp = secp256k1::Secp256k1::new();
        let xpub = xkey.into_xpub(network.into(), &secp);

        Ok((xprv, xpub))
    }

    pub fn type_(&self) -> CosignerType {
        self.type_
    }

    pub fn email_address(&self) -> &Option<EmailAddress> {
        &self.email_address
    }

    pub fn uuid(&self) -> Option<&str> {
        self.uuid.as_deref()
    }

    pub fn xprv(&self) -> &Option<bip32::ExtendedPrivKey> {
        &self.xprv
    }

    pub fn xpub(&self) -> &bip32::ExtendedPubKey {
        &self.xpub
    }

    pub fn wallet(&self) -> &Option<Uuid> {
        &self.wallet
    }

    pub fn set_wallet(&mut self, uuid: Uuid) -> Result<&Uuid, Box<dyn Error>> {
        match &self.wallet {
            Some(_) => Err("wallet has already been set".into()),
            None => {
                self.wallet = Some(uuid);
                Ok(self.wallet.as_ref().unwrap())
            }
        }
    }

    pub fn remove(&mut self, connection: &mut SqliteConnection) -> Result<(), Box<dyn Error>> {
        if let Some(uuid) = &self.uuid {
            db::Cosigner::remove(connection, uuid)?;
        }
        self.uuid = None;

        Ok(())
    }

    pub fn save(&mut self, connection: &mut SqliteConnection) -> Result<(), Box<dyn Error>> {
        let mut new_record = db::Cosigner::new(
            self.type_,
            self.email_address.as_ref(),
            self.xprv.as_ref(),
            &self.xpub,
            self.wallet.as_ref(),
        );

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
