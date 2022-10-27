use std::str::FromStr;
use std::sync::Mutex;

use bdk::bitcoin::util::bip32;
use diesel::prelude::SqliteConnection;
use email_address::EmailAddress;
use int_enum::IntEnum;
use tonic::transport::{server::Router, Channel, Server};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::db;
use crate::{AddressType, Config, Cosigner, CosignerType, Network, Wallet};

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

        let email_address = EmailAddress::from_str(&inner.email_address)
            .map_err(|_| Status::invalid_argument("invalid email address"))?;

        let xpub = bip32::ExtendedPubKey::from_str(&inner.xpub)
            .map_err(|_| Status::invalid_argument("invalid xpub"))?;

        let cosigner = Cosigner::new(
            CosignerType::External,
            Some(email_address),
            Some(xpub),
            None,
        )
        .map_err(|err| Status::internal(&err.to_string()))?;

        let record = cosigner
            .store(&mut connection)
            .map_err(|err| Status::internal(&err.to_string()))?;

        Ok(Response::new(proto::RegisterCosignerResponse {
            cosigner: Some(record.into()),
        }))
    }

    async fn get_cosigner(
        &self,
        request: Request<proto::GetCosignerRequest>,
    ) -> Result<Response<proto::GetCosignerResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let cosigner_id = request.into_inner().cosigner_id;

        let uuid =
            Uuid::from_str(&cosigner_id).map_err(|_| Status::invalid_argument("invalid UUID"))?;

        let mut records = Cosigner::fetch(&mut connection, Some(uuid), None, None)
            .map_err(|err| Status::internal(&err.to_string()))?;

        let mut cosigner = None;
        if !records.is_empty() {
            let record = records.remove(0);
            cosigner = Some(record.into())
        }

        Ok(Response::new(proto::GetCosignerResponse { cosigner }))
    }

    async fn find_cosigner(
        &self,
        request: Request<proto::FindCosignerRequest>,
    ) -> Result<Response<proto::FindCosignerResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let inner = request.into_inner();

        let email_address = inner
            .email_address
            .map(|address| EmailAddress::from_str(&address))
            .transpose()
            .map_err(|_| Status::invalid_argument("invalid email address"))?;

        let xpub = inner
            .xpub
            .map(|xpub| bip32::ExtendedPubKey::from_str(&xpub))
            .transpose()
            .map_err(|_| Status::invalid_argument("invalid xpub"))?;

        let records = Cosigner::fetch(&mut connection, None, email_address, xpub)
            .map_err(|err| Status::internal(&err.to_string()))?;

        let mut cosigners = vec![];
        for record in records {
            cosigners.push(record.into())
        }

        Ok(Response::new(proto::FindCosignerResponse { cosigners }))
    }

    async fn forget_cosigner(
        &self,
        request: Request<proto::ForgetCosignerRequest>,
    ) -> Result<Response<proto::ForgetCosignerResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let cosigner_id = request.into_inner().cosigner_id;

        let uuid =
            Uuid::from_str(&cosigner_id).map_err(|_| Status::invalid_argument("invalid UUID"))?;

        Cosigner::remove(&mut connection, uuid)
            .map_err(|err| Status::not_found(&err.to_string()))?;

        Ok(Response::new(proto::ForgetCosignerResponse { cosigner_id }))
    }

    async fn create_wallet(
        &self,
        request: Request<proto::CreateWalletRequest>,
    ) -> Result<Response<proto::CreateWalletResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let inner = request.into_inner();

        let address_type = AddressType::from_int(inner.address_type as i16)
            .map_err(|_| Status::invalid_argument("invalid address type"))?;

        let network = Network::from_int(inner.network as i16)
            .map_err(|_| Status::invalid_argument("invalid network"))?;

        if inner.required_sigs < 1 {
            return Err(Status::invalid_argument("required signers should be > 0"));
        }

        let mut cosigner_ids = vec![];
        for id in inner.cosigner_ids {
            cosigner_ids.push(
                Uuid::from_str(id.as_ref())
                    .map_err(|_| Status::invalid_argument("invalid UUID"))?,
            );
        }

        if cosigner_ids.is_empty() {
            return Err(Status::invalid_argument("No valid cosigner_ids"));
        }

        let mut wallet = Wallet::new(
            &mut connection,
            address_type,
            network,
            inner.required_sigs,
            cosigner_ids,
        )
        .map_err(|err| Status::internal(&err.to_string()))?;

        let record = wallet
            .store(&mut connection)
            .map_err(|err| Status::internal(&err.to_string()))?;

        Ok(Response::new(proto::CreateWalletResponse {
            wallet: Some(record.into()),
        }))
    }

    async fn get_wallet(
        &self,
        request: Request<proto::GetWalletRequest>,
    ) -> Result<Response<proto::GetWalletResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();

        let mut records = Wallet::fetch(
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
                crate::AddressType::from_int(type_ as i16)
                    .map_err(|_| Status::internal("invalid address type"))?,
            );
        }

        let network = inner
            .network
            .map(|network| Network::from_int(network as i16))
            .transpose()
            .map_err(|_| Status::internal("invalid network"))?;

        let records = Wallet::fetch(&mut connection, None, address_type, network)
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
