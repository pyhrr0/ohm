use std::sync::Mutex;

use diesel::prelude::SqliteConnection;
use int_enum::IntEnum;
use tonic::transport::{server::Router, Channel, Server};
use tonic::{Request, Response, Status};

use crate::db;
use crate::Config;

use super::proto;
use proto::ohm_api_client as grpc_client;
use proto::ohm_api_server as grpc_server;

pub struct Servicer {
    db_connection: Mutex<SqliteConnection>,
    _config: Config,
}

#[tonic::async_trait]
impl grpc_server::OhmApi for Servicer {
    async fn register_cosigner(
        &self,
        request: Request<proto::RegisterCosignerRequest>,
    ) -> Result<Response<proto::RegisterCosignerResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let inner = request.into_inner();

        let cosigner = db::Cosigner::new(
            db::CosignerType::External,
            &inner.email_address,
            None,
            &inner.xpub,
        );

        let record = cosigner
            .store(&mut connection)
            .map_err(|err| Status::internal(&err.to_string()))?;

        Ok(Response::new(proto::RegisterCosignerResponse {
            cosigner: Some(proto::Cosigner {
                cosigner_id: record.uuid,
                email_address: record.email_address,
                xpub: record.xpub,
                wallet_id: None,
            }),
        }))
    }

    async fn get_cosigner(
        &self,
        request: Request<proto::GetCosignerRequest>,
    ) -> Result<Response<proto::GetCosignerResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();

        let mut records = db::Cosigner::fetch(
            &mut connection,
            Some(&request.into_inner().cosigner_id),
            None,
            None,
        )
        .map_err(|err| Status::internal(&err.to_string()))?;

        let mut cosigner = None;
        if !records.is_empty() {
            let record = records.remove(0);

            cosigner = Some(proto::Cosigner {
                cosigner_id: record.uuid,
                email_address: record.email_address,
                xpub: record.xpub,
                wallet_id: record.wallet_uuid,
            });
        }

        Ok(Response::new(proto::GetCosignerResponse { cosigner }))
    }

    async fn find_cosigner(
        &self,
        request: Request<proto::FindCosignerRequest>,
    ) -> Result<Response<proto::FindCosignerResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let inner = request.into_inner();

        let records = db::Cosigner::fetch(
            &mut connection,
            None,
            inner.email_address.as_deref(),
            inner.xpub.as_deref(),
        )
        .map_err(|err| Status::internal(&err.to_string()))?;

        let mut cosigners = vec![];
        for record in records {
            cosigners.push(proto::Cosigner {
                cosigner_id: record.uuid,
                email_address: record.email_address,
                xpub: record.xpub,
                wallet_id: record.wallet_uuid,
            });
        }

        Ok(Response::new(proto::FindCosignerResponse { cosigners }))
    }

    async fn forget_cosigner(
        &self,
        request: Request<proto::ForgetCosignerRequest>,
    ) -> Result<Response<proto::ForgetCosignerResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let cosigner_id = request.into_inner().cosigner_id;

        db::Cosigner::remove(&mut connection, &cosigner_id)
            .map_err(|err| Status::internal(&err.to_string()))?;

        Ok(Response::new(proto::ForgetCosignerResponse { cosigner_id }))
    }

    async fn create_wallet(
        &self,
        _request: Request<proto::CreateWalletRequest>,
    ) -> Result<Response<proto::CreateWalletResponse>, Status> {
        unimplemented!()
    }

    async fn get_wallet(
        &self,
        request: Request<proto::GetWalletRequest>,
    ) -> Result<Response<proto::GetWalletResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();

        let mut records = db::Wallet::fetch(
            &mut connection,
            Some(&request.into_inner().wallet_id),
            None,
            None,
        )
        .map_err(|err| Status::internal(&err.to_string()))?;

        let mut wallet = None;
        if !records.is_empty() {
            let record = records.remove(0);

            wallet = Some(proto::Wallet {
                wallet_id: record.uuid,
                balance: record.balance.to_string(),
                receive_descriptor: record.receive_descriptor,
                receive_address: record.receive_address,
                receive_address_index: record.receive_address_index as u64,
                change_descriptor: record.change_descriptor,
                change_address: record.change_address,
                change_address_index: record.change_address_index as u64,
                transactions: vec![proto::Transaction {}], // TODO
            });
        }

        Ok(Response::new(proto::GetWalletResponse { wallet }))
    }

    async fn find_wallet(
        &self,
        request: Request<proto::FindWalletRequest>,
    ) -> Result<Response<proto::FindWalletResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let inner = request.into_inner();

        let mut address_type = None;
        if let Some(type_) = inner.address_type {
            address_type = Some(
                db::AddressType::from_int(type_ as i16)
                    .map_err(|_| Status::internal("Invalid address type"))?,
            );
        }

        let mut network = None;
        if let Some(net) = inner.network {
            network = Some(
                db::Network::from_int(net as i16)
                    .map_err(|_| Status::internal("Invalid network"))?,
            );
        }

        let records = db::Wallet::fetch(&mut connection, None, address_type, network)
            .map_err(|err| Status::internal(&err.to_string()))?;

        let mut wallets = vec![];
        for record in records {
            wallets.push(proto::Wallet {
                wallet_id: record.uuid,
                receive_descriptor: record.receive_descriptor,
                receive_address: record.receive_address,
                receive_address_index: record.receive_address_index as u64,
                change_descriptor: record.change_descriptor,
                change_address: record.change_address,
                change_address_index: record.change_address_index as u64,
                balance: record.balance.to_string(),
                transactions: vec![proto::Transaction {}], // TODO
            });
        }

        Ok(Response::new(proto::FindWalletResponse { wallets }))
    }

    async fn get_receive_address(
        &self,
        _request: Request<proto::GetReceiveAddressRequest>,
    ) -> Result<Response<proto::GetReceiveAddressResponse>, Status> {
        unimplemented!()
    }

    async fn forget_wallet(
        &self,
        request: Request<proto::ForgetWalletRequest>,
    ) -> Result<Response<proto::ForgetWalletResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let wallet_id = request.into_inner().wallet_id;

        db::Psbt::remove(&mut connection, &wallet_id)
            .map_err(|err| Status::internal(&err.to_string()))?;

        Ok(Response::new(proto::ForgetWalletResponse { wallet_id }))
    }

    async fn create_psbt(
        &self,
        _request: Request<proto::CreatePsbtRequest>,
    ) -> Result<Response<proto::CreatePsbtResponse>, Status> {
        unimplemented!()
    }

    async fn register_psbt(
        &self,
        _request: Request<proto::RegisterPsbtRequest>,
    ) -> Result<Response<proto::RegisterPsbtResponse>, Status> {
        unimplemented!()
    }

    async fn get_psbt(
        &self,
        _request: Request<proto::GetPsbtRequest>,
    ) -> Result<Response<proto::GetPsbtResponse>, Status> {
        unimplemented!()
    }

    async fn find_psbt(
        &self,
        _request: Request<proto::FindPsbtRequest>,
    ) -> Result<Response<proto::FindPsbtResponse>, Status> {
        unimplemented!()
    }

    async fn sign_psbt(
        &self,
        _request: Request<proto::SignPsbtRequest>,
    ) -> Result<Response<proto::SignPsbtResponse>, Status> {
        unimplemented!()
    }

    async fn combine_with_other_psbt(
        &self,
        _request: Request<proto::CombineWithOtherPsbtRequest>,
    ) -> Result<Response<proto::CombineWithOtherPsbtResponse>, Status> {
        unimplemented!()
    }

    async fn broadcast_psbt(
        &self,
        _request: Request<proto::BroadcastPsbtRequest>,
    ) -> Result<Response<proto::BroadcastPsbtResponse>, Status> {
        unimplemented!()
    }

    async fn forget_psbt(
        &self,
        request: Request<proto::ForgetPsbtRequest>,
    ) -> Result<Response<proto::ForgetPsbtResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let psbt_id = request.into_inner().psbt_id;

        db::Psbt::remove(&mut connection, &psbt_id)
            .map_err(|err| Status::internal(&err.to_string()))?;

        Ok(Response::new(proto::ForgetPsbtResponse { psbt_id }))
    }
}

impl Servicer {
    pub fn new(
        config: Config,
    ) -> Result<
        Router<grpc_server::OhmApiServer<Servicer>, tonic::transport::server::Unimplemented>,
        Box<dyn std::error::Error>,
    > {
        Ok(
            Server::builder().add_service(grpc_server::OhmApiServer::new(Servicer {
                db_connection: Mutex::new(db::establish_connection(
                    &config.db_path.to_string_lossy(),
                )),
                _config: config,
            })),
        )
    }
}

pub type Client = grpc_client::OhmApiClient<Channel>;

impl Client {
    pub async fn attach(endpoint: &str) -> Result<Client, Box<dyn std::error::Error>> {
        let channel = Channel::from_shared(endpoint.to_string())?;
        let client = grpc_client::OhmApiClient::new(channel.connect().await?);

        Ok(client)
    }
}

#[derive(Debug)]
pub enum OhmResponse {
    RegisterCosigner(Response<proto::RegisterCosignerResponse>),
    GetCosigner(Response<proto::GetCosignerResponse>),
    FindCosigner(Response<proto::FindCosignerResponse>),
    ForgetCosigner(Response<proto::ForgetCosignerResponse>),
    CreateWallet(Response<proto::CreateWalletResponse>),
    GetWallet(Response<proto::GetWalletResponse>),
    FindWallet(Response<proto::FindWalletResponse>),
    ForgetWallet(Response<proto::ForgetWalletResponse>),
    GetReceiveAddress(Response<proto::GetReceiveAddressResponse>),
    CreatePsbt(Response<proto::CreatePsbtResponse>),
    RegisterPsbt(Response<proto::RegisterPsbtResponse>),
    GetPsbt(Response<proto::GetPsbtResponse>),
    FindPsbt(Response<proto::FindPsbtResponse>),
    SignPsbt(Response<proto::SignPsbtResponse>),
    CombineWithOtherPsbt(Response<proto::CombineWithOtherPsbtResponse>),
    BroadcastPsbt(Response<proto::BroadcastPsbtResponse>),
    ForgetPsbt(Response<proto::ForgetPsbtResponse>),
}
