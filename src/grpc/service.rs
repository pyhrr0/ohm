use std::{str::FromStr, sync::Mutex};

use bdk::{
    bitcoin::{psbt::PartiallySignedTransaction, util::bip32, Address},
    descriptor::DescriptorPublicKey,
};
use diesel::SqliteConnection;
use email_address::EmailAddress;
use int_enum::IntEnum;
use rust_decimal::Decimal;
use tonic::{
    transport::{server::Router, Channel, Server},
    Request, Response, Status,
};
use uuid::Uuid;

use super::proto;
use crate::db;
use crate::{AddressType, Config, Cosigner, CosignerType, Network, Psbt, Wallet};
use proto::{ohm_api_client as grpc_client, ohm_api_server as grpc_server};

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

        let mut cosigner = Cosigner::new(
            CosignerType::External,
            Some(email_address),
            Some(xpub),
            None,
        )
        .map_err(|_| Status::internal("failed to create cosigner"))?;

        cosigner
            .save(&mut connection)
            .map_err(|_| Status::internal("failed to register cosigner"))?;

        Ok(Response::new(proto::RegisterCosignerResponse {
            cosigner: Some(cosigner.into()),
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

        let cosigner = Cosigner::from_db(&mut connection, Some(uuid))
            .map_err(|_| Status::internal("failed to enumerate cosigners"))?
            .map(|cosigner| cosigner.into());

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

        let mut results = Cosigner::find(&mut connection, None, email_address, xpub, None)
            .map_err(|_| Status::internal("failed to enumerate cosigners"))?;

        let mut cosigners = vec![];
        for i in 0..results.len() {
            cosigners.push(results.remove(i).into());
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

        let mut cosigner = Cosigner::from_db(&mut connection, Some(uuid))
            .map_err(|_| Status::internal("failed to enumerate cosigners"))?
            .ok_or_else(|| Status::not_found("cosigner could not be found"))?;

        cosigner
            .remove(&mut connection)
            .map_err(|_| Status::internal("failed to remove cosigner"))?;

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
        .map_err(|_| Status::internal("failed to create wallet"))?;

        wallet
            .save(&mut connection)
            .map_err(|_| Status::internal("wallet could not be saved"))?;

        Ok(Response::new(proto::CreateWalletResponse {
            wallet: Some(wallet.into()),
        }))
    }

    async fn get_wallet(
        &self,
        request: Request<proto::GetWalletRequest>,
    ) -> Result<Response<proto::GetWalletResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let wallet_id = request.into_inner().wallet_id;

        let uuid =
            Uuid::from_str(&wallet_id).map_err(|_| Status::invalid_argument("invalid UUID"))?;

        let wallet = Wallet::from_db(&mut connection, Some(uuid))
            .map_err(|_| Status::internal("failed to enumerate wallets"))?
            .map(|wallet| wallet.into());

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
                AddressType::from_int(type_ as i16)
                    .map_err(|_| Status::invalid_argument("invalid address type"))?,
            );
        }

        let network = inner
            .network
            .map(|network| Network::from_int(network as i16))
            .transpose()
            .map_err(|_| Status::invalid_argument("invalid network"))?;

        inner
            .descriptor
            .as_ref()
            .map(|descriptor| DescriptorPublicKey::from_str(descriptor))
            .transpose()
            .map_err(|_| Status::invalid_argument("invalid receive descriptor"))?;

        let results = Wallet::find(
            &mut connection,
            None,
            address_type,
            network,
            inner.descriptor.as_deref(),
        )
        .map_err(|_| Status::internal("failed to enumerate wallets"))?;

        let mut wallets = vec![];
        for result in results {
            wallets.push(result.into());
        }

        Ok(Response::new(proto::FindWalletResponse { wallets }))
    }

    async fn get_new_receive_address(
        &self,
        request: Request<proto::GetNewReceiveAddressRequest>,
    ) -> Result<Response<proto::GetNewReceiveAddressResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let wallet_id = request.into_inner().wallet_id;

        let uuid =
            Uuid::from_str(&wallet_id).map_err(|_| Status::invalid_argument("invalid UUID"))?;

        let mut wallet = Wallet::from_db(&mut connection, Some(uuid))
            .map_err(|_| Status::internal("failed to enumerate wallets"))?
            .ok_or_else(|| Status::not_found("wallet could not be found"))?;

        let address = wallet.new_receive_address().map_err(|err| {
            Status::internal(format!("unable to get new receive address: {}", err))
        })?;

        wallet
            .save(&mut connection)
            .map_err(|_| Status::internal("wallet could not be saved"))?;

        Ok(Response::new(proto::GetNewReceiveAddressResponse {
            address: address.to_string(),
        }))
    }

    async fn forget_wallet(
        &self,
        request: Request<proto::ForgetWalletRequest>,
    ) -> Result<Response<proto::ForgetWalletResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let wallet_id = request.into_inner().wallet_id;

        let uuid =
            Uuid::from_str(&wallet_id).map_err(|_| Status::invalid_argument("invalid UUID"))?;

        let mut wallet = Wallet::from_db(&mut connection, Some(uuid))
            .map_err(|_| Status::internal("failed to enumerate wallets"))?
            .ok_or_else(|| Status::not_found("wallet could not be found"))?;

        wallet
            .remove(&mut connection)
            .map_err(|_| Status::internal("failed to remove wallet"))?;

        Ok(Response::new(proto::ForgetWalletResponse { wallet_id }))
    }

    async fn create_psbt(
        &self,
        request: Request<proto::CreatePsbtRequest>,
    ) -> Result<Response<proto::CreatePsbtResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let inner = request.into_inner();

        let uuid = Uuid::from_str(&inner.wallet_id)
            .map_err(|_| Status::invalid_argument("invalid UUID"))?;

        let amount = Decimal::from_str(&inner.amount)
            .map_err(|_| Status::invalid_argument("invalid amount"))?;

        let recipient = Address::from_str(&inner.recipient)
            .map_err(|_| Status::invalid_argument("invalid recipient"))?;

        let mut wallet = Wallet::from_db(&mut connection, Some(uuid))
            .map_err(|_| Status::internal("failed to enumerate wallets"))?
            .ok_or_else(|| Status::not_found("wallet could not be found"))?;

        let psbt = wallet
            .create_psbt(&mut connection, amount, recipient)
            .map_err(|_| Status::internal("failed to create a PSBT"))?;

        Ok(Response::new(proto::CreatePsbtResponse {
            psbt: Some(psbt.into()),
        }))
    }

    async fn register_psbt(
        &self,
        request: Request<proto::RegisterPsbtRequest>,
    ) -> Result<Response<proto::RegisterPsbtResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let inner = request.into_inner();

        let uuid = Uuid::from_str(&inner.wallet_id)
            .map_err(|_| Status::invalid_argument("invalid UUID"))?;

        let psbt = PartiallySignedTransaction::from_str(&inner.base64)
            .map_err(|_| Status::invalid_argument("invalid PSBT"))?;

        let mut wallet = Wallet::from_db(&mut connection, Some(uuid))
            .map_err(|_| Status::internal("failed to enumerate wallets"))?
            .ok_or_else(|| Status::not_found("wallet could not be found"))?;

        let psbt = wallet
            .import_psbt(&mut connection, psbt)
            .map_err(|_| Status::internal("failed to register PSBT"))?;

        Ok(Response::new(proto::RegisterPsbtResponse {
            psbt: Some(psbt.into()),
        }))
    }

    async fn get_psbt(
        &self,
        request: Request<proto::GetPsbtRequest>,
    ) -> Result<Response<proto::GetPsbtResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let psbt_id = request.into_inner().psbt_id;

        let uuid =
            Uuid::from_str(&psbt_id).map_err(|_| Status::invalid_argument("invalid UUID"))?;

        let psbt = Psbt::from_db(&mut connection, Some(uuid))
            .map_err(|_| Status::internal("failed to enumerate PSBTs"))?
            .map(|psbt| (&psbt).into());

        Ok(Response::new(proto::GetPsbtResponse { psbt }))
    }

    async fn find_psbt(
        &self,
        request: Request<proto::FindPsbtRequest>,
    ) -> Result<Response<proto::FindPsbtResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let wallet_id = request.into_inner().wallet_id;

        let uuid =
            Uuid::from_str(&wallet_id).map_err(|_| Status::invalid_argument("invalid UUID"))?;

        let mut results = Psbt::find(&mut connection, None, Some(uuid))
            .map_err(|_| Status::internal("failed to enumerate PSBTs"))?;

        let mut psbts = vec![];
        for i in 0..results.len() {
            psbts.push((&results.remove(i)).into());
        }

        Ok(Response::new(proto::FindPsbtResponse { psbts }))
    }

    async fn sign_psbt(
        &self,
        request: Request<proto::SignPsbtRequest>,
    ) -> Result<Response<proto::SignPsbtResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let psbt_id = request.into_inner().psbt_id;

        let uuid =
            Uuid::from_str(&psbt_id).map_err(|_| Status::invalid_argument("invalid UUID"))?;

        let psbt = Psbt::from_db(&mut connection, Some(uuid))
            .map_err(|_| Status::internal("failed to enumerate PSBTs"))?
            .ok_or_else(|| Status::not_found("PSBT could not be found"))?;

        let mut wallet = Wallet::from_db(&mut connection, Some(*psbt.wallet()))
            .map_err(|_| Status::internal("failed to enumerate wallets"))?
            .ok_or_else(|| Status::not_found("wallet could not be found"))?;

        let signed_psbt = wallet
            .sign_psbt(&mut connection, uuid)
            .map_err(|_| Status::internal("failed to sign PSBT"))?;

        Ok(Response::new(proto::SignPsbtResponse {
            psbt: Some(signed_psbt.into()),
        }))
    }

    async fn combine_with_other_psbt(
        &self,
        request: Request<proto::CombineWithOtherPsbtRequest>,
    ) -> Result<Response<proto::CombineWithOtherPsbtResponse>, Status> {
        let mut connection = self.db_connection.lock().unwrap();
        let inner = request.into_inner();

        let uuid =
            Uuid::from_str(&inner.psbt_id).map_err(|_| Status::invalid_argument("invalid UUID"))?;

        let psbt = Psbt::from_db(&mut connection, Some(uuid))
            .map_err(|_| Status::internal("failed to enumerate PSBTs"))?
            .ok_or_else(|| Status::not_found("PSBT could not be found"))?;

        let additional_psbt = PartiallySignedTransaction::from_str(&inner.base64)
            .map_err(|_| Status::invalid_argument("invalid PSBT"))?;

        let mut wallet = Wallet::from_db(&mut connection, Some(*psbt.wallet()))
            .map_err(|_| Status::internal("failed to enumerate wallets"))?
            .ok_or_else(|| Status::not_found("wallet could not be found"))?;

        let combined_psbt = wallet
            .combine_psbt(&mut connection, uuid, additional_psbt)
            .map_err(|_| Status::internal("failed to combine PSBTs"))?;

        Ok(Response::new(proto::CombineWithOtherPsbtResponse {
            psbt: Some(combined_psbt.into()),
        }))
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

        let uuid =
            Uuid::from_str(&psbt_id).map_err(|_| Status::invalid_argument("invalid UUID"))?;

        let mut psbt = Psbt::from_db(&mut connection, Some(uuid))
            .map_err(|_| Status::internal("failed to enumerate PSBTs"))?
            .ok_or_else(|| Status::not_found("PSBT could not be found"))?;

        psbt.remove(&mut connection)
            .map_err(|_| Status::internal("failed to remove cosigner"))?;

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
    GetNewReceiveAddress(Response<proto::GetNewReceiveAddressResponse>),
    CreatePsbt(Response<proto::CreatePsbtResponse>),
    RegisterPsbt(Response<proto::RegisterPsbtResponse>),
    GetPsbt(Response<proto::GetPsbtResponse>),
    FindPsbt(Response<proto::FindPsbtResponse>),
    SignPsbt(Response<proto::SignPsbtResponse>),
    CombineWithOtherPsbt(Response<proto::CombineWithOtherPsbtResponse>),
    BroadcastPsbt(Response<proto::BroadcastPsbtResponse>),
    ForgetPsbt(Response<proto::ForgetPsbtResponse>),
}
