use bdk::bitcoin;
use tonic::include_proto;

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

impl From<crate::Cosigner> for Cosigner {
    fn from(cosigner: crate::Cosigner) -> Self {
        Self {
            cosigner_id: cosigner
                .uuid()
                .map_or(String::from(""), |uuid| uuid.to_string()),
            email_address: cosigner
                .email_address()
                .as_ref()
                .map_or(String::from(""), |email| email.to_string()),
            xpub: cosigner.xpub().to_string(),
            wallet_id: cosigner.wallet().map(|uuid| uuid.to_string()),
        }
    }
}

impl From<crate::Wallet> for Wallet {
    fn from(wallet: crate::Wallet) -> Self {
        Self {
            wallet_id: wallet
                .uuid()
                .map_or(String::from(""), |uuid| uuid.to_string()),
            required_sigs: wallet.required_signatures(),
            balance: wallet.balance().unwrap().confirmed.to_string(),
            descriptor: String::from(wallet.receive_descriptor()),
            receive_address: wallet.receive_address().unwrap().to_string(),
            transactions: vec![Transaction {}], // TODO
        }
    }
}

impl From<&crate::Psbt> for Psbt {
    fn from(psbt: &crate::Psbt) -> Self {
        Self {
            psbt_id: psbt
                .uuid()
                .map_or(String::from(""), |uuid| uuid.to_string()),
            base64: psbt.base64().to_string(),
            wallet_id: psbt.wallet().to_string(),
        }
    }
}
