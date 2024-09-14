use std::net::{IpAddr, Ipv4Addr};
use crab_nat::{natpmp::*, InternetProtocol};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Specify the gateway's IP (usually your router)
    let gateway_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));  // Replace with your actual gateway IP

    // Step 1: Request public (external) IP address with default timeout config
    let public_address = try_external_address(gateway_ip, None).await?;
    println!("External IP Address: {}", public_address);

    // Step 2: Add a port mapping (TCP on port 8080 for 3600 seconds)
    // let mapping = try_port_mapping(gateway_ip, InternetProtocol::Tcp, 8080, 8080, 3600, None).await?;
    // println!("Port mapped: External {} -> Internal {}", mapping.public_port(), mapping.internal_port());

    Ok(())
}