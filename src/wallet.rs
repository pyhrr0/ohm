use std::error::Error;
use std::str::FromStr;

use bdk::{
    bitcoin::{util::bip32, Address},
    blockchain::ElectrumBlockchain,
    database::MemoryDatabase,
    descriptor,
    electrum_client::Client,
    keys::{IntoDescriptorKey, ScriptContext},
    wallet::AddressIndex,
    Balance, SyncOptions,
};
use diesel::SqliteConnection;
use uuid::Uuid;

use super::{Cosigner, CosignerType};
use crate::{db, db::WalletDescriptors};
pub use db::{AddressType, Network};

#[derive(Debug)]
enum ExtendedKey {
    PrivKey((bip32::ExtendedPrivKey, bip32::DerivationPath)),
    PubKey((bip32::ExtendedPubKey, bip32::DerivationPath)),
}

impl<Ctx: ScriptContext> IntoDescriptorKey<Ctx> for ExtendedKey {
    fn into_descriptor_key(self) -> Result<bdk::keys::DescriptorKey<Ctx>, bdk::keys::KeyError> {
        match self {
            ExtendedKey::PrivKey(xprv) => xprv.into_descriptor_key(),
            ExtendedKey::PubKey(xpub) => xpub.into_descriptor_key(),
        }
    }
}

pub struct Wallet {
    uuid: Option<String>,
    address_type: AddressType,
    network: Network,
    required_signatures: u64,
    descriptors: WalletDescriptors,
    receive_address_index: u64,
    change_address_index: u64,
    bdk_handle: bdk::Wallet<MemoryDatabase>,
    internal_cosigner: Cosigner,
}

impl Wallet {
    pub fn new(
        connection: &mut SqliteConnection,
        address_type: AddressType,
        network: Network,
        required_signatures: u64,
        cosigners: Vec<Uuid>,
    ) -> Result<Self, Box<dyn Error>> {
        let cosigner = Cosigner::new(CosignerType::Internal, None, None, Some(network))?;
        let xpubs = Self::get_xpubs(connection, cosigners)?;

        let (receive_descriptor, receive_descriptor_watch_only) = Self::create_descriptor(
            address_type,
            required_signatures as usize,
            bip32::DerivationPath::from_str("m/0").unwrap(),
            cosigner.xprv(),
            &xpubs,
        )?;

        let (change_descriptor, change_descriptor_watch_only) = Self::create_descriptor(
            address_type,
            required_signatures as usize,
            bip32::DerivationPath::from_str("m/1").unwrap(),
            cosigner.xprv(),
            &xpubs,
        )?;

        let bdk_handle =
            Self::initialize_bdk_handle(&receive_descriptor, &change_descriptor, network)?;

        Ok(Self {
            uuid: None,
            address_type,
            network,
            required_signatures,
            descriptors: WalletDescriptors {
                receive_descriptor, // TODO encrypt
                receive_descriptor_watch_only,
                change_descriptor, // TODO encrypt
                change_descriptor_watch_only,
            },
            receive_address_index: 0,
            change_address_index: 0,
            internal_cosigner: cosigner,
            bdk_handle,
        })
    }

    pub fn from_db(
        connection: &mut SqliteConnection,
        uuid: Option<Uuid>,
    ) -> Result<Option<Self>, Box<dyn Error>> {
        let mut wallets = Self::find(connection, uuid, None, None, None)?;

        Ok(match !wallets.is_empty() {
            true => Some(wallets.remove(0)),
            false => None,
        })
    }

    pub fn find(
        connection: &mut SqliteConnection,
        uuid: Option<Uuid>,
        address_type: Option<AddressType>,
        network: Option<Network>,
        receive_descriptor: Option<&str>,
    ) -> Result<Vec<Self>, Box<dyn Error>> {
        let records = db::Wallet::find(
            connection,
            uuid.as_ref(),
            address_type,
            network,
            receive_descriptor,
        )?;

        let mut wallets = vec![];
        for record in records {
            let cosigner = Cosigner::from_db(connection, Some(Uuid::from_str(&record.uuid)?))?
                .ok_or("associated internal cosigner could not be found")?;

            let bdk_handle = Self::initialize_bdk_handle(
                &record.receive_descriptor, // TODO decrypt
                &record.change_descriptor,  // TODO decrypt
                record.network,
            )?;

            wallets.push(Wallet {
                uuid: Some(record.uuid),
                address_type: record.address_type,
                network: record.network,
                required_signatures: record.required_signatures as u64,
                descriptors: WalletDescriptors {
                    receive_descriptor: record.receive_descriptor,
                    receive_descriptor_watch_only: record.receive_descriptor_watch_only,
                    change_descriptor: record.change_descriptor,
                    change_descriptor_watch_only: record.change_descriptor_watch_only,
                },
                receive_address_index: record.receive_address_index as u64,
                change_address_index: record.change_address_index as u64,
                internal_cosigner: cosigner,
                bdk_handle,
            });
        }

        Ok(wallets)
    }

    fn create_descriptor(
        address_type: AddressType,
        required_signers: usize,
        derivation_path: bip32::DerivationPath,
        xprv: &Option<bip32::ExtendedPrivKey>,
        xpubs: &Vec<bip32::ExtendedPubKey>,
    ) -> Result<(String, String), Box<dyn Error>> {
        let mut keys = vec![];
        if let Some(xprv) = xprv {
            keys.push(ExtendedKey::PrivKey((*xprv, derivation_path.clone())));
        }

        for xpub in xpubs {
            keys.push(ExtendedKey::PubKey((*xpub, derivation_path.clone())));
        }

        let descriptor = match address_type {
            AddressType::P2sh => descriptor!(sh(sortedmulti_vec(required_signers, keys))),
            AddressType::P2wsh => descriptor!(wsh(sortedmulti_vec(required_signers, keys))),
            AddressType::P2shwsh => {
                descriptor!(sh(wsh(sortedmulti_vec(required_signers, keys))))
            }
        }?;

        Ok((
            descriptor.0.to_string_with_secret(&descriptor.1),
            descriptor.0.to_string(),
        ))
    }

    fn get_xpubs(
        connection: &mut SqliteConnection,
        cosigner_ids: Vec<Uuid>,
    ) -> Result<Vec<bip32::ExtendedPubKey>, Box<dyn Error>> {
        let mut xpubs = vec![];
        for uuid in cosigner_ids {
            let records = db::Cosigner::find(connection, Some(&uuid), None, None, None)?;
            let cosigner = records.get(0).ok_or_else(|| -> Box<dyn Error> {
                format!("cosigner could not be found: {}", uuid).into()
            })?;
            xpubs.push(bip32::ExtendedPubKey::from_str(cosigner.xpub.as_ref())?);
        }

        Ok(xpubs)
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

    pub fn address_type(&self) -> AddressType {
        self.address_type
    }

    pub fn balance(&self) -> Result<Balance, Box<dyn Error>> {
        Ok(self.bdk_handle.get_balance()?)
    }

    pub fn network(&self) -> Network {
        self.network
    }

    pub fn receive_descriptor(&self) -> &str {
        &self.descriptors.receive_descriptor_watch_only
    }

    pub fn receive_address_index(&self) -> u64 {
        self.receive_address_index
    }

    pub fn receive_address(&self) -> Result<Address, Box<dyn Error>> {
        Ok(self
            .bdk_handle
            .get_address(AddressIndex::Peek(self.receive_address_index as u32))?
            .address)
    }

    pub fn change_descriptor(&self) -> &str {
        &self.descriptors.change_descriptor_watch_only
    }

    pub fn change_address_index(&self) -> u64 {
        self.change_address_index
    }

    pub fn change_address(&self) -> Result<Address, Box<dyn Error>> {
        Ok(self
            .bdk_handle
            .get_address(AddressIndex::Peek(self.change_address_index as u32))?
            .address)
    }

    pub fn new_receive_address(&mut self) -> Result<Address, Box<dyn Error>> {
        self.receive_address_index += 1;
        self.receive_address()
    }

    pub fn new_change_address(&mut self) -> Result<Address, Box<dyn Error>> {
        self.change_address_index += 1;
        self.change_address()
    }

    pub fn required_signatures(&self) -> u64 {
        self.required_signatures
    }

    pub fn uuid(&self) -> Option<&str> {
        self.uuid.as_deref()
    }

    pub fn remove(&mut self, connection: &mut SqliteConnection) -> Result<(), Box<dyn Error>> {
        if let Some(uuid) = &self.uuid {
            db::Wallet::remove(connection, uuid)?;
        }
        self.uuid = None;

        Ok(())
    }

    pub fn save(&mut self, connection: &mut SqliteConnection) -> Result<(), Box<dyn Error>> {
        let mut new_record = db::Wallet::new(
            self.address_type,
            self.network,
            self.required_signatures as i16,
            &self.balance()?,
            &self.descriptors, // TODO encrypt
            self.receive_address_index as i64,
            self.change_address_index as i64,
        );

        if let Some(uuid) = &self.uuid {
            new_record.uuid = uuid.clone();
        };

        let record = new_record.upsert(connection)?;

        if self.uuid.is_none() {
            self.internal_cosigner
                .set_wallet(Uuid::from_str(&record.uuid)?)?;
            self.internal_cosigner.save(connection)?;
            self.uuid = Some(record.uuid)
        }

        Ok(())
    }
}
