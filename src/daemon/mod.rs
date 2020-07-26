use tonic::{transport::Server, Request, Response, Status};

use crate::container_manager::{ContainerManager, ContainerOptions};

use cruise_grpc::cruise_server::{Cruise, CruiseServer};
use cruise_grpc::{CreateContainerRequest, CreateContainerResponse};

mod cruise_grpc {
    tonic::include_proto!("cruise"); // The string specified here must match the proto package name
}

#[derive(Debug)]
pub struct CruiseDaemon {
    cm: ContainerManager,
}

impl CruiseDaemon {
    #[tokio::main]
    pub async fn run_server(self: Self, port: &str) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("[::1]:{}", port).parse()?;

        Server::builder()
            .add_service(CruiseServer::new(self))
            .serve(addr)
            .await?;

        Ok(())
    }
}

pub fn new(cm: ContainerManager) -> CruiseDaemon {
    return CruiseDaemon { cm };
}

#[tonic::async_trait]
impl Cruise for CruiseDaemon {
    async fn create_container(
        &self,
        request: Request<CreateContainerRequest>,
    ) -> Result<Response<CreateContainerResponse>, Status> {
        println!("Got a request: {:?}", request);

        let request = request.into_inner();
        let container_opts = ContainerOptions {
            name: request.name,
            command: request.command,
            args: request.args,
            rootfs_path: request.rootfs_path,
        };

        match self.cm.create_container(container_opts) {
            Ok(container_id) => Ok(Response::new(CreateContainerResponse { container_id })),
            Err(err) => Err(Status::new(tonic::Code::Internal, format!("{:?}", err))),
        }
    }
}
