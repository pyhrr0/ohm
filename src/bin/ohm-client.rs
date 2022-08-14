use bdk::bitcoin::{Address, AddressType, Network, PublicKey};
use email_address::EmailAddress;
use structopt::{clap::AppSettings, StructOpt};
use url::Url;
use uuid::Uuid;

use ohm::create_client;

#[derive(Debug, StructOpt)]
enum CosignerOpts {
    Register {
        _email: EmailAddress,
        _pubkey: PublicKey,
    },
    Info {
        _cosigner_id: Uuid,
    },
    List {
        _email: Option<EmailAddress>,
        _pubkey: Option<PublicKey>,
    },
}

#[derive(Debug, StructOpt)]
enum WalletOpts {
    Create {
        _address_type: String, // TODO use AddressType
        _network: Network,
        _required_sigs: u64,
        #[structopt(required = true)]
        _cosigner_ids: Vec<Uuid>,
    },
    Info {
        _wallet_id: Uuid,
    },
    List {
        _address_type: Option<String>, // TODO use AddressType
        _network: Option<Network>,
    },
}

#[derive(Debug, StructOpt)]
enum PsbtOpts {
    Create {
        _wallet_id: Uuid,
        _amount: String,
        _address: Address,
    },
    Import {
        _wallet_id: Uuid,
        _psbt: String,
    },
    Sign {
        _wallet_id: Uuid,
        _psbt_id: Uuid,
    },
    Combine {
        _wallet_id: Uuid,
        _psbt_id: Uuid,
        _psbt: String,
    },
    Broadcast {
        _wallet_id: Uuid,
        _psbt_id: Uuid,
    },
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(display_order = 0)]
    Cosigner(CosignerOpts),

    #[structopt(display_order = 1)]
    Wallet(WalletOpts),

    #[structopt(display_order = 2)]
    Psbt(PsbtOpts),
}

#[derive(StructOpt, Debug)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
struct Opts {
    #[structopt(short, long, default_value = "http://127.0.0.1:1234")]
    endpoint: Url,

    #[structopt(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::from_args();
    let _client = create_client(opts.endpoint.as_str()).await?;
    dbg!(&opts);

    match opts.command {
        Command::Cosigner(opts) => match opts {
            CosignerOpts::Register { _email, _pubkey } => {
                unimplemented!();
            }
            CosignerOpts::Info { _cosigner_id } => {
                unimplemented!();
            }
            CosignerOpts::List { _email, _pubkey } => {
                unimplemented!();
            }
        },

        Command::Wallet(opts) => match opts {
            WalletOpts::Create {
                _address_type,
                _network,
                _required_sigs,
                _cosigner_ids,
            } => {
                unimplemented!();
            }
            WalletOpts::Info { _wallet_id } => {
                unimplemented!();
            }
            WalletOpts::List {
                _address_type,
                _network,
            } => {
                unimplemented!();
            }
        },

        Command::Psbt(opts) => match opts {
            PsbtOpts::Create {
                _wallet_id,
                _amount,
                _address,
            } => {
                unimplemented!()
            }
            PsbtOpts::Import { _wallet_id, _psbt } => {
                unimplemented!()
            }
            PsbtOpts::Sign {
                _wallet_id,
                _psbt_id,
            } => {
                unimplemented!()
            }
            PsbtOpts::Combine {
                _wallet_id,
                _psbt_id,
                _psbt,
            } => {
                unimplemented!()
            }
            PsbtOpts::Broadcast {
                _wallet_id,
                _psbt_id,
            } => {
                unimplemented!()
            }
        },
    }
}
