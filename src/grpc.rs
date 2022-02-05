use tonic::transport::{server::Router, Channel, Server};
use tonic::{Request, Response, Status};

use crate::Config;

mod pb {
    tonic::include_proto!("ohm.v1");
}
use pb::ohm_api_client as grpc_client;
use pb::ohm_api_server as grpc_server;

pub struct Servicer {
    _config: Config,
}

#[tonic::async_trait]
impl grpc_server::OhmApi for Servicer {
    async fn get_signer_info(
        &self,
        _request: Request<pb::GetSignerInfoRequest>,
    ) -> Result<Response<pb::GetSignerInfoResponse>, Status> {
        unimplemented!()
    }

    async fn register_signer(
        &self,
        _request: Request<pb::RegisterSignerRequest>,
    ) -> Result<Response<pb::RegisterSignerResponse>, Status> {
        unimplemented!()
    }

    async fn find_signer(
        &self,
        _request: Request<pb::FindSignerRequest>,
    ) -> Result<Response<pb::FindSignerResponse>, Status> {
        unimplemented!()
    }

    async fn create_wallet(
        &self,
        _request: Request<pb::CreateWalletRequest>,
    ) -> Result<Response<pb::CreateWalletResponse>, Status> {
        unimplemented!()
    }

    async fn get_wallet_info(
        &self,
        _request: Request<pb::GetWalletInfoRequest>,
    ) -> Result<Response<pb::GetWalletInfoResponse>, Status> {
        unimplemented!()
    }

    async fn get_receive_address(
        &self,
        _request: Request<pb::GetReceiveAddressRequest>,
    ) -> Result<Response<pb::GetReceiveAddressResponse>, Status> {
        unimplemented!()
    }

    async fn create_psbt(
        &self,
        _request: Request<pb::CreatePsbtRequest>,
    ) -> Result<Response<pb::CreatePsbtResponse>, Status> {
        unimplemented!()
    }

    async fn import_psbt(
        &self,
        _request: Request<pb::ImportPsbtRequest>,
    ) -> Result<Response<pb::ImportPsbtResponse>, Status> {
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
}

pub async fn create_client(
    endpoint: &str,
) -> Result<grpc_client::OhmApiClient<Channel>, Box<dyn std::error::Error>> {
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
    Ok(Server::builder().add_service(grpc_server::OhmApiServer::new(Servicer { _config: config })))
}
