use chrono::offset::Utc;
use chrono::DateTime;
use tonic::{transport::Server, Request, Response, Status};

use crate::container::Container;
use crate::container_manager::{ContainerManager, ContainerOptions};

use cruise_grpc::cruise_server::{Cruise, CruiseServer};
use cruise_grpc::{
    CreateContainerRequest, CreateContainerResponse, GetContainerRequest, GetContainerResponse,
    ListContainersRequest, ListContainersResponse, StartContainerRequest, StartContainerResponse,
};

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

    async fn start_container(
        &self,
        request: Request<StartContainerRequest>,
    ) -> Result<Response<StartContainerResponse>, Status> {
        println!("Got a request: {:?}", request);

        let request = request.into_inner();

        match self.cm.start_container(&request.container_id) {
            Ok(_) => Ok(Response::new(StartContainerResponse { success: true })),
            Err(err) => Err(Status::new(tonic::Code::Internal, format!("{:?}", err))),
        }
    }

    async fn get_container(
        &self,
        request: Request<GetContainerRequest>,
    ) -> Result<Response<GetContainerResponse>, Status> {
        println!("Got a request: {:?}", request);

        let request = request.into_inner();

        match self.cm.get_container(&request.container_id) {
            Ok(container) => Ok(Response::new(map_container_to_container_response(
                *container,
            ))),
            Err(err) => Err(Status::new(tonic::Code::Internal, format!("{:?}", err))),
        }
    }

    async fn list_containers(
        &self,
        request: Request<ListContainersRequest>,
    ) -> Result<Response<ListContainersResponse>, Status> {
        println!("Got a request: {:?}", request);

        match self.cm.list_containers() {
            Ok(containers) => Ok(Response::new(ListContainersResponse {
                containers: containers
                    .into_iter()
                    .map(map_container_to_container_response)
                    .collect(),
            })),
            Err(err) => Err(Status::new(tonic::Code::Internal, format!("{:?}", err))),
        }
    }
}

fn map_container_to_container_response(container: Container) -> GetContainerResponse {
    GetContainerResponse {
        id: container.id,
        name: container.name,
        status: container.status.to_string(),
        exit_code: container.exit_code,
        created_at: match container.created_at {
            Some(created_at) => {
                let datetime: DateTime<Utc> = created_at.into();
                format!("{}", datetime.format("%+"))
            }
            None => "Not created yet.".into(),
        },
    }
}
