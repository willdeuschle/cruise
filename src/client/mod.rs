use crate::container::ID;
use cruise_grpc::cruise_client::CruiseClient;
use cruise_grpc::{
    CreateContainerRequest, DeleteContainerRequest, GetContainerRequest, GetContainerResponse,
    ListContainersRequest, StartContainerRequest, StopContainerRequest,
};
use log::debug;
use std::cmp::max;

mod cruise_grpc {
    tonic::include_proto!("cruise");
}

#[tokio::main]
pub async fn create_container(
    port: &str,
    container_name: &str,
    command: &str,
    args: Vec<String>,
    rootfs_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = CruiseClient::connect(format!("http://[::1]:{}", port)).await?;

    let request = tonic::Request::new(CreateContainerRequest {
        name: container_name.into(),
        command: command.into(),
        args,
        rootfs_path: rootfs_path.into(),
    });

    let response = client.create_container(request).await?;

    debug!("Got create container response: {:?}", response);

    Ok(())
}

#[tokio::main]
pub async fn start_container(
    port: &str,
    container_id: ID,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = CruiseClient::connect(format!("http://[::1]:{}", port)).await?;

    let request = tonic::Request::new(StartContainerRequest { container_id });

    let response = client.start_container(request).await?;

    debug!("Got start container response: {:?}", response);

    Ok(())
}

#[tokio::main]
pub async fn stop_container(
    port: &str,
    container_id: ID,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = CruiseClient::connect(format!("http://[::1]:{}", port)).await?;

    let request = tonic::Request::new(StopContainerRequest { container_id });

    let response = client.stop_container(request).await?;

    debug!("Got stop container response: {:?}", response);

    Ok(())
}

#[tokio::main]
pub async fn get_container(port: &str, container_id: ID) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = CruiseClient::connect(format!("http://[::1]:{}", port)).await?;

    let request = tonic::Request::new(GetContainerRequest { container_id });

    let response = client.get_container(request).await?;

    debug!("Got get container response: {:?}", response);

    print_containers(vec![response.into_inner()]);

    Ok(())
}

#[tokio::main]
pub async fn list_containers(port: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = CruiseClient::connect(format!("http://[::1]:{}", port)).await?;

    let request = tonic::Request::new(ListContainersRequest {});

    let response = client.list_containers(request).await?;

    debug!("Got list containers response: {:?}", response);

    print_containers(response.into_inner().containers);

    Ok(())
}

#[tokio::main]
pub async fn delete_container(
    port: &str,
    container_id: ID,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = CruiseClient::connect(format!("http://[::1]:{}", port)).await?;

    let request = tonic::Request::new(DeleteContainerRequest { container_id });

    let response = client.delete_container(request).await?;

    debug!("Got delete container response: {:?}", response);

    Ok(())
}

fn print_containers(containers: Vec<GetContainerResponse>) {
    let id_column = "ID";
    let mut id_len = id_column.len();

    let name_column = "NAME";
    let mut name_len = name_column.len();

    let status_column = "STATUS";
    let mut status_len = status_column.len();

    let exit_code_column = "EXIT_CODE";
    let mut exit_code_len = exit_code_column.len();

    let created_at_column = "CREATED_AT";
    let mut created_at_len = created_at_column.len();

    let started_at_column = "STARTED_AT";
    let mut started_at_len = started_at_column.len();

    let finished_at_column = "FINISHED_AT";
    let mut finished_at_len = finished_at_column.len();

    let command_column = "COMMAND";
    let mut command_len = command_column.len();

    let args_column = "ARGS";
    let mut args_len = args_column.len();

    for container in containers.iter() {
        id_len = max(id_len, container.id.len());
        name_len = max(name_len, container.name.len());
        status_len = max(status_len, container.status.len());
        exit_code_len = max(exit_code_len, format!("{}", container.exit_code).len());
        created_at_len = max(created_at_len, container.created_at.len());
        started_at_len = max(started_at_len, container.started_at.len());
        finished_at_len = max(finished_at_len, container.finished_at.len());
        command_len = max(command_len, container.command.len());
        args_len = max(args_len, container.args.len());
    }

    println!(
        "{:<id$} {:<name$} {:<status$} {:<exit_code$} {:<created_at$} {:<started_at$} {:<finished_at$} {:<command$} {:<args$}",
        id_column,
        name_column,
        status_column,
        exit_code_column,
        created_at_column,
        started_at_column,
        finished_at_column,
        command_column,
        args_column,
        id = id_len,
        name = name_len,
        status = status_len,
        exit_code = exit_code_len,
        created_at = created_at_len,
        started_at = started_at_len,
        finished_at = finished_at_len,
        command = command_len,
        args = args_len,
    );
    for container in containers.iter() {
        println!(
            "{:<id$} {:<name$} {:<status$} {:<exit_code$} {:<created_at$} {:<started_at$} {:<finished_at$} {:<command$} {:<args$}",
            container.id,
            container.name,
            container.status,
            container.exit_code,
            container.created_at,
            container.started_at,
            container.finished_at,
            container.command,
            container.args.join(","),
            id = id_len,
            name = name_len,
            status = status_len,
            exit_code = exit_code_len,
            created_at = created_at_len,
            started_at = started_at_len,
            finished_at = finished_at_len,
            command = command_len,
            args = args_len,
        );
    }
}
