use bdk::bitcoin;
use tonic::include_proto;

include_proto!("ohm.v1");

impl From<&str> for AddressType {
    fn from(address_type: &str) -> Self {
        match address_type {
            "sh" => AddressType::P2sh,
            "wsh" => AddressType::P2wsh,
            "sh_wsh" => AddressType::P2shwsh,
            "tr" => AddressType::P2tr,
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
