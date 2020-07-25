use cruise_grpc::cruise_client::CruiseClient;
use cruise_grpc::CreateContainerRequest;

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

    println!("RESPONSE={:?}", response);

    Ok(())
}
