use bdk::bitcoin;
use tonic::include_proto;

use crate::db;

include_proto!("ohm.v1");

impl From<&str> for AddressType {
    fn from(address_type: &str) -> Self {
        match address_type {
            "sh" => AddressType::P2sh,
            "wsh" => AddressType::P2wsh,
            "sh_wsh" => AddressType::P2shwsh,
            _ => {
                panic!("proto contains an unsupported address type")
            }
        }
    }
}

impl From<bitcoin::Network> for Network {
    fn from(network: bitcoin::Network) -> Self {
        match network {
            bitcoin::Network::Bitcoin => Network::Mainnet,
            bitcoin::Network::Testnet => Network::Testnet,
            bitcoin::Network::Regtest => Network::Regtest,
            _ => {
                panic!("proto contains an unsupported network type")
            }
        }
    }
}

impl From<db::CosignerRecord> for Cosigner {
    fn from(record: db::CosignerRecord) -> Self {
        Cosigner {
            cosigner_id: record.uuid,
            email_address: record.email_address,
            xpub: record.xpub,
            wallet_id: record.wallet_uuid,
        }
    }
}
