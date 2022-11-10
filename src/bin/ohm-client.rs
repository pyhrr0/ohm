use std::{error::Error, fmt::Debug};

use bdk::bitcoin::{util::bip32::ExtendedPubKey, Address, Network};
use email_address::EmailAddress;
use structopt::{clap::AppSettings, StructOpt};
use tonic::Request;
use url::Url;
use uuid::Uuid;

use ohm::{proto, Client, Response};

#[derive(Debug, StructOpt)]
enum CosignerOptions {
    Register {
        email_address: EmailAddress,
        xpub: ExtendedPubKey,
    },
    Info {
        cosigner_id: Uuid,
    },
    Find {
        email_address: Option<EmailAddress>,
        xpub: Option<ExtendedPubKey>,
    },
    Forget {
        cosigner_id: Uuid,
    },
}

#[derive(Debug, StructOpt)]
enum WalletOptions {
    Create {
        address_type: String, // TODO use AddressType
        network: Network,
        required_sigs: u64,
        #[structopt(required = true)]
        cosigner_ids: Vec<Uuid>,
    },
    Info {
        wallet_id: Uuid,
    },
    Find {
        address_type: Option<String>, // TODO use AddressType
        network: Option<Network>,
        descriptor: Option<String>,
    },
    Forget {
        wallet_id: Uuid,
    },
}

#[derive(Debug, StructOpt)]
enum PsbtOptions {
    Create {
        wallet_id: Uuid,
        amount: String,
        recipient: Address,
    },
    Register {
        wallet_id: Uuid,
        psbt: String,
    },
    Info {
        psbt_id: Uuid,
    },
    Find {
        wallet_id: Uuid,
    },
    Sign {
        psbt_id: Uuid,
    },
    Combine {
        psbt_id: Uuid,
        psbt: String,
    },
    Broadcast {
        psbt_id: Uuid,
    },
    Forget {
        psbt_id: Uuid,
    },
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(display_order = 0)]
    Cosigner(CosignerOptions),

    #[structopt(display_order = 1)]
    Wallet(WalletOptions),

    #[structopt(display_order = 2)]
    Psbt(PsbtOptions),
}

#[derive(StructOpt, Debug)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
struct Options {
    #[structopt(short, long, default_value = "http://127.0.0.1:1234")]
    endpoint: Url,

    #[structopt(subcommand)]
    command: Command,
}

async fn handle_cosigner_requests(
    client: &mut Client,
    options: &CosignerOptions,
) -> Result<Response, Box<dyn Error>> {
    match options {
        CosignerOptions::Register {
            email_address,
            xpub,
        } => {
            let request = Request::new(proto::RegisterCosignerRequest {
                email_address: email_address.to_string(),
                xpub: xpub.to_string(),
            });
            Ok(Response::RegisterCosigner(
                client.register_cosigner(request).await?,
            ))
        }

        CosignerOptions::Info { cosigner_id } => {
            let request = Request::new(proto::GetCosignerRequest {
                cosigner_id: cosigner_id.to_string(),
            });
            Ok(Response::GetCosigner(client.get_cosigner(request).await?))
        }

        CosignerOptions::Find {
            email_address,
            xpub,
        } => {
            let request = Request::new(proto::FindCosignerRequest {
                email_address: email_address.as_ref().map(|email| email.to_string()),
                xpub: xpub.map(|xpub| xpub.to_string()),
            });
            Ok(Response::FindCosigner(client.find_cosigner(request).await?))
        }

        CosignerOptions::Forget { cosigner_id } => {
            let request = Request::new(proto::ForgetCosignerRequest {
                cosigner_id: cosigner_id.to_string(),
            });
            Ok(Response::ForgetCosigner(
                client.forget_cosigner(request).await?,
            ))
        }
    }
}
async fn handle_wallet_requests(
    client: &mut Client,
    options: &WalletOptions,
) -> Result<Response, Box<dyn Error>> {
    match options {
        WalletOptions::Create {
            address_type,
            network,
            required_sigs,
            cosigner_ids,
        } => {
            let cosigners = cosigner_ids.iter().map(|uuid| uuid.to_string()).collect();
            let request = Request::new(proto::CreateWalletRequest {
                address_type: proto::AddressType::from(address_type.as_str()).into(),
                network: proto::Network::from(*network).into(),
                required_sigs: *required_sigs,
                cosigner_ids: cosigners,
            });
            Ok(Response::CreateWallet(client.create_wallet(request).await?))
        }

        WalletOptions::Info { wallet_id } => {
            let request = Request::new(proto::GetWalletRequest {
                wallet_id: wallet_id.to_string(),
            });
            Ok(Response::GetWallet(client.get_wallet(request).await?))
        }

        WalletOptions::Find {
            address_type,
            network,
            descriptor,
        } => {
            let request = Request::new(proto::FindWalletRequest {
                address_type: address_type
                    .clone()
                    .map(|address_type| proto::AddressType::from(address_type.as_str()).into()),
                network: network.map(|network| proto::Network::from(network).into()),
                descriptor: descriptor.clone(),
            });
            Ok(Response::FindWallet(client.find_wallet(request).await?))
        }

        WalletOptions::Forget { wallet_id } => {
            let request = Request::new(proto::ForgetWalletRequest {
                wallet_id: wallet_id.to_string(),
            });
            Ok(Response::ForgetWallet(client.forget_wallet(request).await?))
        }
    }
}

async fn handle_psbt_requests(
    client: &mut Client,
    options: &PsbtOptions,
) -> Result<Response, Box<dyn Error>> {
    match options {
        PsbtOptions::Create {
            wallet_id,
            amount,
            recipient,
        } => {
            let request = Request::new(proto::CreatePsbtRequest {
                wallet_id: wallet_id.to_string(),
                amount: amount.clone(),
                recipient: recipient.to_string(),
            });
            Ok(Response::CreatePsbt(client.create_psbt(request).await?))
        }

        PsbtOptions::Register { wallet_id, psbt } => {
            let request = Request::new(proto::RegisterPsbtRequest {
                wallet_id: wallet_id.to_string(),
                base64: psbt.clone(),
            });
            Ok(Response::RegisterPsbt(client.register_psbt(request).await?))
        }

        PsbtOptions::Info { psbt_id } => {
            let request = Request::new(proto::GetPsbtRequest {
                psbt_id: psbt_id.to_string(),
            });
            Ok(Response::GetPsbt(client.get_psbt(request).await?))
        }

        PsbtOptions::Find { wallet_id } => {
            let request = Request::new(proto::FindPsbtRequest {
                wallet_id: wallet_id.to_string(),
            });
            Ok(Response::FindPsbt(client.find_psbt(request).await?))
        }

        PsbtOptions::Sign { psbt_id } => {
            let request = Request::new(proto::SignPsbtRequest {
                psbt_id: psbt_id.to_string(),
            });
            Ok(Response::SignPsbt(client.sign_psbt(request).await?))
        }

        PsbtOptions::Combine { psbt_id, psbt } => {
            let request = Request::new(proto::CombineWithOtherPsbtRequest {
                psbt_id: psbt_id.to_string(),
                base64: psbt.clone(),
            });
            Ok(Response::CombineWithOtherPsbt(
                client.combine_with_other_psbt(request).await?,
            ))
        }

        PsbtOptions::Broadcast { psbt_id } => {
            let request = Request::new(proto::BroadcastPsbtRequest {
                psbt_id: psbt_id.to_string(),
            });
            Ok(Response::BroadcastPsbt(
                client.broadcast_psbt(request).await?,
            ))
        }

        PsbtOptions::Forget { psbt_id } => {
            let request = Request::new(proto::ForgetPsbtRequest {
                psbt_id: psbt_id.to_string(),
            });
            Ok(Response::ForgetPsbt(client.forget_psbt(request).await?))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opts = Options::from_args();
    let mut client = Client::attach(opts.endpoint.as_str()).await?;

    let response = match opts.command {
        Command::Cosigner(opts) => handle_cosigner_requests(&mut client, &opts).await?,
        Command::Wallet(opts) => handle_wallet_requests(&mut client, &opts).await?,
        Command::Psbt(opts) => handle_psbt_requests(&mut client, &opts).await?,
    };
    println!("RESPONSE={:?}", response);
    Ok(())
}
