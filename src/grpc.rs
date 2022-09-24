use std::sync::Mutex;

use bdk::bitcoin;
use diesel::prelude::SqliteConnection;
use tonic::transport::{server::Router, Channel, Server};
use tonic::{Request, Response, Status};

use crate::db;
use crate::Config;

pub mod pb {
    #![allow(clippy::derive_partial_eq_without_eq)]
    tonic::include_proto!("ohm.v1");
}

use pb::ohm_api_client as grpc_client;
use pb::ohm_api_server as grpc_server;

pub struct Servicer {
    db_conn: Mutex<SqliteConnection>,
    _config: Config,
}

impl From<&str> for pb::AddressType {
    fn from(address_type: &str) -> Self {
        match address_type {
            "sh" => pb::AddressType::P2sh,
            "wsh" => pb::AddressType::P2wsh,
            "sh_wsh" => pb::AddressType::P2shwsh,
            "tr" => pb::AddressType::P2tr,
            _ => {
                panic!("Received an unsupported network")
            }
        }
    }
}

impl From<bitcoin::Network> for pb::Network {
    fn from(network: bitcoin::Network) -> Self {
        match network {
            bitcoin::Network::Bitcoin => pb::Network::Mainnet,
            bitcoin::Network::Testnet => pb::Network::Testnet,
            bitcoin::Network::Regtest => pb::Network::Regtest,
            _ => {
                panic!("Received an unsupported network")
            }
        }
    }
}

#[derive(Debug)]
pub enum OhmResponse {
    RegisterCosigner(Response<pb::RegisterCosignerResponse>),
    GetCosigner(Response<pb::GetCosignerResponse>),
    FindCosigner(Response<pb::FindCosignerResponse>),
    ForgetCosigner(Response<pb::ForgetCosignerResponse>),
    CreateWallet(Response<pb::CreateWalletResponse>),
    GetWallet(Response<pb::GetWalletResponse>),
    FindWallet(Response<pb::FindWalletResponse>),
    ForgetWallet(Response<pb::ForgetWalletResponse>),
    GetReceiveAddress(Response<pb::GetReceiveAddressResponse>),
    CreatePsbt(Response<pb::CreatePsbtResponse>),
    RegisterPsbt(Response<pb::RegisterPsbtResponse>),
    SignPsbt(Response<pb::SignPsbtResponse>),
    CombineWithOtherPsbt(Response<pb::CombineWithOtherPsbtResponse>),
    BroadcastPsbt(Response<pb::BroadcastPsbtResponse>),
    ForgetPsbt(Response<pb::ForgetPsbtResponse>),
}

#[tonic::async_trait]
impl grpc_server::OhmApi for Servicer {
    async fn register_cosigner(
        &self,
        request: Request<pb::RegisterCosignerRequest>,
    ) -> Result<Response<pb::RegisterCosignerResponse>, Status> {
        let mut conn = self.db_conn.lock().unwrap();
        let inner = request
            .into_inner()
            .cosigner
            .ok_or_else(|| Status::invalid_argument("Cosigner field should be set"))?;

        let cosigner = db::Cosigner::new(
            &inner.email_address,
            &inner.public_key,
            db::CosignerType::External,
        );

        let record = cosigner
            .store(&mut conn)
            .map_err(|err| Status::internal(&err.to_string()))?;

        Ok(Response::new(pb::RegisterCosignerResponse {
            cosigner_id: record.uuid,
        }))
    }

    async fn get_cosigner(
        &self,
        _request: Request<pb::GetCosignerRequest>,
    ) -> Result<Response<pb::GetCosignerResponse>, Status> {
        unimplemented!()
    }

    async fn find_cosigner(
        &self,
        _request: Request<pb::FindCosignerRequest>,
    ) -> Result<Response<pb::FindCosignerResponse>, Status> {
        unimplemented!()
    }

    async fn forget_cosigner(
        &self,
        _request: Request<pb::ForgetCosignerRequest>,
    ) -> Result<Response<pb::ForgetCosignerResponse>, Status> {
        unimplemented!()
    }

    async fn create_wallet(
        &self,
        _request: Request<pb::CreateWalletRequest>,
    ) -> Result<Response<pb::CreateWalletResponse>, Status> {
        unimplemented!()
    }

    async fn get_wallet(
        &self,
        _request: Request<pb::GetWalletRequest>,
    ) -> Result<Response<pb::GetWalletResponse>, Status> {
        unimplemented!()
    }

    async fn find_wallet(
        &self,
        _request: Request<pb::FindWalletRequest>,
    ) -> Result<Response<pb::FindWalletResponse>, Status> {
        unimplemented!()
    }

    async fn get_receive_address(
        &self,
        _request: Request<pb::GetReceiveAddressRequest>,
    ) -> Result<Response<pb::GetReceiveAddressResponse>, Status> {
        unimplemented!()
    }

    async fn forget_wallet(
        &self,
        _request: Request<pb::ForgetWalletRequest>,
    ) -> Result<Response<pb::ForgetWalletResponse>, Status> {
        unimplemented!()
    }

    async fn create_psbt(
        &self,
        _request: Request<pb::CreatePsbtRequest>,
    ) -> Result<Response<pb::CreatePsbtResponse>, Status> {
        unimplemented!()
    }

    async fn register_psbt(
        &self,
        _request: Request<pb::RegisterPsbtRequest>,
    ) -> Result<Response<pb::RegisterPsbtResponse>, Status> {
        unimplemented!()
    }

    async fn sign_psbt(
        &self,
        _request: Request<pb::SignPsbtRequest>,
    ) -> Result<Response<pb::SignPsbtResponse>, Status> {
        unimplemented!()
    }

    async fn combine_with_other_psbt(
        &self,
        _request: Request<pb::CombineWithOtherPsbtRequest>,
    ) -> Result<Response<pb::CombineWithOtherPsbtResponse>, Status> {
        unimplemented!()
    }

    async fn broadcast_psbt(
        &self,
        _request: Request<pb::BroadcastPsbtRequest>,
    ) -> Result<Response<pb::BroadcastPsbtResponse>, Status> {
        unimplemented!()
    }

    async fn forget_psbt(
        &self,
        _request: Request<pb::ForgetPsbtRequest>,
    ) -> Result<Response<pb::ForgetPsbtResponse>, Status> {
        unimplemented!()
    }
}

pub type Client = grpc_client::OhmApiClient<Channel>;

pub async fn create_client(endpoint: &str) -> Result<Client, Box<dyn std::error::Error>> {
    let channel = Channel::from_shared(endpoint.to_string())?;
    let client = grpc_client::OhmApiClient::new(channel.connect().await?);

    Ok(client)
}

pub fn create_server(
    config: Config,
) -> Result<
    Router<grpc_server::OhmApiServer<Servicer>, tonic::transport::server::Unimplemented>,
    Box<dyn std::error::Error>,
> {
    Ok(
        Server::builder().add_service(grpc_server::OhmApiServer::new(Servicer {
            db_conn: Mutex::new(db::establish_connection(&config.db_path.to_string_lossy())),
            _config: config,
        })),
    )
}
