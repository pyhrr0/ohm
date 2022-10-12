use std::error::Error;

use bdk::bitcoin::util::bip32;
use bdk::bitcoin::{secp256k1, Network};
use bdk::keys::{bip39, DerivableKey, ExtendedKey};
use diesel::prelude::SqliteConnection;
use email_address::EmailAddress;
use uuid::Uuid;

use crate::db;

pub struct Cosigner(db::Cosigner);

pub enum CosignerType {
    Internal = 1,
    External = 2,
}

impl Cosigner {
    pub fn new(
        type_: CosignerType,
        email_address: EmailAddress,
        xpub: Option<bip32::ExtendedPubKey>,
        network: Option<Network>,
    ) -> Result<Cosigner, Box<dyn Error>> {
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

        Ok(Self(db::Cosigner::new(
            type_.into(),
            email_address,
            xprv,
            xpub,
        )))
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
        let xprv: bip32::ExtendedPrivKey = xkey.into_xprv(network).unwrap();

        let xkey: ExtendedKey = mnemonic.into_extended_key()?;
        let secp = secp256k1::Secp256k1::new();
        let xpub = xkey.into_xpub(network, &secp);

        Ok((xprv, xpub))
    }

    pub fn store(
        &self,
        connection: &mut SqliteConnection,
    ) -> Result<db::CosignerRecord, Box<dyn Error>> {
        self.0.store(connection)
    }

    pub fn fetch(
        connection: &mut SqliteConnection,
        uuid: Option<Uuid>,
        email_address: Option<EmailAddress>,
        xpub: Option<bip32::ExtendedPubKey>,
    ) -> Result<Vec<db::CosignerRecord>, Box<dyn Error>> {
        db::Cosigner::fetch(connection, uuid, email_address, xpub)
    }

    pub fn remove(connection: &mut SqliteConnection, uuid: Uuid) -> Result<usize, Box<dyn Error>> {
        db::Cosigner::remove(connection, uuid)
    }
}
