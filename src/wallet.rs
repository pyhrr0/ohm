use std::error::Error;
use std::str::FromStr;

use bdk::bitcoin::util::bip32;
use bdk::blockchain::ElectrumBlockchain;
use bdk::database::MemoryDatabase;
use bdk::electrum_client::Client;
use bdk::keys::{IntoDescriptorKey, ScriptContext};
use bdk::{descriptor, SyncOptions};
use diesel::SqliteConnection;
use uuid::Uuid;

use crate::db;
pub use db::{AddressType, Network};

use super::{Cosigner, CosignerType};

#[derive(Debug)]
enum ExtendedKeyWrapper {
    PrivKey((bip32::ExtendedPrivKey, bip32::DerivationPath)),
    PubKey((bip32::ExtendedPubKey, bip32::DerivationPath)),
}

impl<Ctx: ScriptContext> IntoDescriptorKey<Ctx> for ExtendedKeyWrapper {
    fn into_descriptor_key(self) -> Result<bdk::keys::DescriptorKey<Ctx>, bdk::keys::KeyError> {
        match self {
            ExtendedKeyWrapper::PrivKey(pk) => pk.into_descriptor_key(),
            ExtendedKeyWrapper::PubKey(pk) => pk.into_descriptor_key(),
        }
    }
}

pub struct Wallet {
    internal_cosigner: Cosigner,
    _bdk_handle: bdk::Wallet<MemoryDatabase>,
    pub address_type: AddressType,
    pub network: Network,
    pub receive_descriptor: String,
    pub receive_address_index: u64,
    pub change_descriptor: String,
    pub change_address_index: u64,
    pub required_signatures: u64,
}

impl Wallet {
    pub fn new(
        connection: &mut SqliteConnection,
        address_type: AddressType,
        network: Network,
        required_signers: u64,
        cosigners: Vec<Uuid>,
    ) -> Result<Self, Box<dyn Error>> {
        let cosigner = Cosigner::new(CosignerType::Internal, None, None, Some(network))?;
        let xpubs = Self::get_xpubs(connection, cosigners)?;

        let receive_descriptor = Self::create_descriptor(
            address_type,
            required_signers as usize,
            bip32::DerivationPath::from_str("m/0").unwrap(),
            cosigner.xprv,
            &xpubs,
        )?;

        let change_descriptor = Self::create_descriptor(
            address_type,
            required_signers as usize,
            bip32::DerivationPath::from_str("m/1").unwrap(),
            cosigner.xprv,
            &xpubs,
        )?;

        Ok(Self {
            internal_cosigner: cosigner,
            _bdk_handle: Self::load(&receive_descriptor, &change_descriptor, network)?,
            address_type,
            network,
            receive_descriptor,
            receive_address_index: 0,
            change_descriptor,
            change_address_index: 0,
            required_signatures: required_signers,
        })
    }

    fn load(
        receive_descriptor: &str,
        change_descriptor: &str,
        network: Network,
    ) -> Result<bdk::Wallet<MemoryDatabase>, Box<dyn Error>> {
        let wallet = bdk::Wallet::new(
            receive_descriptor,
            Some(change_descriptor),
            network.into(),
            MemoryDatabase::default(),
        )?;

        let blockchain =
            ElectrumBlockchain::from(Client::new("ssl://electrum.blockstream.info:60002")?);
        wallet.sync(&blockchain, SyncOptions::default())?;

        Ok(wallet)
    }

    fn create_descriptor(
        address_type: AddressType,
        required_signers: usize,
        derivation_path: bip32::DerivationPath,
        xprv: Option<bip32::ExtendedPrivKey>,
        xpubs: &Vec<bip32::ExtendedPubKey>,
    ) -> Result<String, Box<dyn Error>> {
        let mut keys = vec![];
        if let Some(xprv) = xprv {
            keys.push(ExtendedKeyWrapper::PrivKey((xprv, derivation_path.clone())));
        }

        for xpub in xpubs {
            keys.push(ExtendedKeyWrapper::PubKey((*xpub, derivation_path.clone())));
        }

        let descriptor = match address_type {
            AddressType::P2sh => descriptor!(sh(sortedmulti_vec(required_signers, keys))),
            AddressType::P2wsh => descriptor!(wsh(sortedmulti_vec(required_signers, keys))),
            AddressType::P2shwsh => {
                descriptor!(sh(wsh(sortedmulti_vec(required_signers, keys))))
            }
        }?;

        Ok(descriptor.0.to_string_with_secret(&descriptor.1))
    }

    fn get_xpubs(
        connection: &mut SqliteConnection,
        cosigner_ids: Vec<Uuid>,
    ) -> Result<Vec<bip32::ExtendedPubKey>, Box<dyn Error>> {
        let mut xpubs = vec![];
        for uuid in cosigner_ids {
            let records = db::Cosigner::fetch(connection, Some(uuid), None, None)?;
            let cosigner = records.get(0).ok_or_else(|| -> Box<dyn Error> {
                format!("cosigner could not be found: {}", uuid).into()
            })?;
            xpubs.push(bip32::ExtendedPubKey::from_str(cosigner.xpub.as_ref())?);
        }

        Ok(xpubs)
    }

    pub fn store(
        &mut self,
        connection: &mut SqliteConnection,
    ) -> Result<db::WalletRecord, Box<dyn Error>> {
        let record = db::Wallet::new(
            self.address_type,
            self.network,
            &self.receive_descriptor, // TODO encrypt
            &self.change_descriptor,  // TODO encrypt
            self.required_signatures as i16,
        )
        .store(connection)?;

        self.internal_cosigner.wallet = Some(Uuid::from_str(&record.uuid)?);
        self.internal_cosigner.store(connection)?;

        Ok(record)
    }

    pub fn fetch(
        connection: &mut SqliteConnection,
        uuid: Option<&str>,
        address_type: Option<AddressType>,
        network: Option<Network>,
    ) -> Result<Vec<db::WalletRecord>, Box<dyn Error>> {
        db::Wallet::fetch(connection, uuid, address_type, network)
    }

    pub fn remove(connection: &mut SqliteConnection, uuid: Uuid) -> Result<usize, Box<dyn Error>> {
        db::Wallet::remove(connection, uuid)
    }
}
