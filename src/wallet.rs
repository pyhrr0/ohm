use std::error::Error;
use std::str::FromStr;

use bdk::bitcoin::util::bip32;
use bdk::bitcoin::Address;
use bdk::blockchain::ElectrumBlockchain;
use bdk::database::MemoryDatabase;
use bdk::electrum_client::Client;
use bdk::keys::{IntoDescriptorKey, ScriptContext};
use bdk::wallet::AddressIndex;
use bdk::{descriptor, SyncOptions};
use diesel::SqliteConnection;
use uuid::Uuid;

use crate::db;
use db::WalletDescriptors;
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
    pub uuid: Uuid,
    pub address_type: AddressType,
    pub network: Network,
    pub required_signatures: u64,
    pub receive_address: Address,
    receive_address_index: u64,
    change_address: Address,
    change_address_index: u64,
    descriptors: WalletDescriptors,
    internal_cosigner: Cosigner,
    bdk_handle: bdk::Wallet<MemoryDatabase>,
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

        let bdk_handle =
            Self::initialize_bdk_handle(&receive_descriptor, &change_descriptor, network)?;

        Ok(Self {
            uuid: Uuid::new_v4(),
            address_type,
            network,
            required_signatures: required_signers,
            receive_address: bdk_handle.get_address(AddressIndex::Peek(0))?.address,
            receive_address_index: 0,
            change_address: bdk_handle
                .get_internal_address(AddressIndex::Peek(0))?
                .address,
            change_address_index: 0,
            descriptors: WalletDescriptors {
                receive_descriptor,
                change_descriptor,
            },
            internal_cosigner: cosigner,
            bdk_handle,
        })
    }

    pub fn load(connection: &mut SqliteConnection, uuid: Uuid) -> Result<Self, Box<dyn Error>> {
        let records = db::Wallet::fetch(connection, Some(&uuid), None, None)?;
        let wallet = records
            .into_iter()
            .next()
            .ok_or_else(|| -> Box<dyn Error> {
                format!("wallet could not be found: {}", uuid).into()
            })?;

        let records = db::Cosigner::fetch(
            connection,
            None,
            None,
            None,
            Some(&Uuid::from_str(&wallet.uuid)?),
        )?;
        let cosigner = records
            .into_iter()
            .next()
            .ok_or_else(|| -> Box<dyn Error> {
                "associated internal cosigner could not be found".into()
            })?;

        let bdk_handle = Self::initialize_bdk_handle(
            &wallet.receive_descriptor,
            &wallet.change_descriptor,
            wallet.network,
        )?;

        Ok(Self {
            uuid: Uuid::from_str(&wallet.uuid)?,
            address_type: wallet.address_type,
            network: wallet.network,
            required_signatures: wallet.required_signatures as u64,
            receive_address: bdk_handle
                .get_address(AddressIndex::Peek(wallet.receive_address_index as u32))?
                .address,
            change_address: bdk_handle
                .get_internal_address(AddressIndex::Peek(wallet.change_address_index as u32))?
                .address,
            receive_address_index: wallet.receive_address_index as u64,
            change_address_index: wallet.change_address_index as u64,
            descriptors: WalletDescriptors {
                receive_descriptor: wallet.receive_descriptor, // TODO decrypt
                change_descriptor: wallet.change_descriptor,   // TODO decrypt
            },
            internal_cosigner: cosigner.into(),
            bdk_handle,
        })
    }

    fn initialize_bdk_handle(
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
            let records = db::Cosigner::fetch(connection, Some(&uuid), None, None, None)?;
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
            &self.uuid,
            self.address_type,
            self.network,
            &self.descriptors,
            self.receive_address_index as i64,
            self.change_address_index as i64,
            self.required_signatures as i16,
        )
        .store(connection)?;

        self.internal_cosigner.wallet = Some(self.uuid);
        self.internal_cosigner.store(connection)?;

        Ok(record)
    }

    pub fn fetch(
        connection: &mut SqliteConnection,
        uuid: Option<&Uuid>,
        address_type: Option<AddressType>,
        network: Option<Network>,
    ) -> Result<Vec<db::WalletRecord>, Box<dyn Error>> {
        db::Wallet::fetch(connection, uuid, address_type, network)
    }

    pub fn remove(connection: &mut SqliteConnection, uuid: Uuid) -> Result<usize, Box<dyn Error>> {
        db::Wallet::remove(connection, uuid)
    }

    pub fn get_new_receive_address(
        &mut self,
        connection: &mut SqliteConnection,
    ) -> Result<&Address, Box<dyn Error>> {
        let receive_address_index = self.receive_address_index + 1;
        let receive_address = self
            .bdk_handle
            .get_address(AddressIndex::Peek(receive_address_index as u32))?
            .address;

        let change_address_index = self.change_address_index + 1;
        let change_address = self
            .bdk_handle
            .get_address(AddressIndex::Peek(change_address_index as u32))?
            .address;

        db::Wallet::update(
            connection,
            &self.uuid,
            Some(&receive_address),
            Some(receive_address_index),
            Some(&change_address),
            Some(change_address_index),
        )?;

        self.receive_address = receive_address;
        self.receive_address_index = receive_address_index;

        self.change_address = change_address;
        self.change_address_index = change_address_index;

        Ok(&self.receive_address)
    }
}
