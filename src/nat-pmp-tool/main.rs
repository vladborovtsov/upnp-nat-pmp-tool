use std::num::{NonZeroU16};
use std::net::{IpAddr, Ipv4Addr};
use crab_nat::{natpmp::*, InternetProtocol, PortMappingOptions};
use default_net::gateway::get_default_gateway;
use std::io::{self, Write};
use default_net::get_interfaces;

fn ipv4_network_address(ip: Ipv4Addr, netmask: Ipv4Addr) -> Ipv4Addr {
    let ip_octets = ip.octets();
    let mask_octets = netmask.octets();
    Ipv4Addr::new(
        ip_octets[0] & mask_octets[0],
        ip_octets[1] & mask_octets[1],
        ip_octets[2] & mask_octets[2],
        ip_octets[3] & mask_octets[3],
    )
}

fn get_internal_ip() -> Result<Ipv4Addr, Box<dyn std::error::Error>> {
    let gateway = match get_default_gateway()?.ip_addr {
        std::net::IpAddr::V4(gateway_ip) => gateway_ip,
        _ => return Err("Gateway is not an IPv4 address.".into()),
    };
    let ifaces = get_interfaces();
    for iface in ifaces {
        for ip in iface.ipv4 {
            if !ip.addr.is_loopback() && ip.addr != Ipv4Addr::new(0, 0, 0, 0) {
                let iface_network = ipv4_network_address(ip.addr, ip.netmask);
                let gateway_network = ipv4_network_address(gateway, ip.netmask);
                if iface_network == gateway_network {
                    println!("Found internal IP: {}", ip.addr);
                    return Ok(ip.addr);
                }
            }
        }
    }

    Err("Could not find a valid interface associated with the default gateway.".into())
}

fn prompt_user<T: std::str::FromStr>(prompt: &str, default: T) -> Result<T, Box<dyn std::error::Error>> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim().is_empty() {
        return Ok(default);
    }

    input.trim().parse::<T>().map_err(|_| "Failed to parse input".into())
}

fn prompt_protocol() -> Result<&'static str, Box<dyn std::error::Error>> {
    println!("Select protocol:");
    println!("1. TCP");
    println!("2. UDP");
    println!("3. BOTH");
    print!("Enter your choice (1/2/3): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim() {
        "1" => Ok("TCP"),
        "2" => Ok("UDP"),
        "3" => Ok("BOTH"),
        _ => Err("Invalid protocol choice".into()),
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    match get_default_gateway() {
        Ok(gateway) => {
            if let std::net::IpAddr::V4(gateway_ip) = gateway.ip_addr {
                println!("Default Gateway as Ipv4Addr: {}", gateway_ip);
                let public_address = try_external_address(IpAddr::from(gateway_ip), None).await?;
                println!("External IP Address: {}", public_address);

                let internal_ip = get_internal_ip()?;
                let internal_port = prompt_user("Enter internal port (default 8080): ", 8080)?;
                let external_port = prompt_user("Enter external port (default 8080): ", 8080)?;
                let protocol = prompt_protocol()?;
                let ttl = prompt_user("Enter TTL in seconds (default 3600): ", 3600)?;

                let internal_port_nonzero = NonZeroU16::new(internal_port).unwrap();
                let external_port_nonzero = NonZeroU16::new(external_port).unwrap();
                let mut port_mapping_options = PortMappingOptions::default();
                port_mapping_options.external_port = Option::from(external_port_nonzero);
                port_mapping_options.lifetime_seconds = Some(ttl);

                match protocol {
                    "TCP" => {
                        let _mapping = try_port_mapping(
                            IpAddr::from(gateway_ip),
                            InternetProtocol::Tcp,
                            internal_port_nonzero,
                            port_mapping_options
                        ).await?;
                        println!(
                            "TCP Port mapping added: External port {} -> {}:{}",
                            external_port, internal_ip, internal_port
                        );
                    }
                    "UDP" => {
                        let _mapping = try_port_mapping(
                            IpAddr::from(gateway_ip),
                            InternetProtocol::Udp,
                            internal_port_nonzero,
                            port_mapping_options
                        ).await?;
                        println!(
                            "UDP Port mapping added: External port {} -> {}:{}",
                            external_port, internal_ip, internal_port
                        );
                    }
                    "BOTH" => {
                        // Add both TCP and UDP mappings
                        let _mapping_tcp = try_port_mapping(
                            IpAddr::from(gateway_ip),
                            InternetProtocol::Tcp,
                            internal_port_nonzero,
                            port_mapping_options
                        ).await?;
                        println!(
                            "TCP Port mapping added: External port {} -> {}:{}",
                            external_port, internal_ip, internal_port
                        );

                        let _mapping_udp = try_port_mapping(
                            IpAddr::from(gateway_ip),
                            InternetProtocol::Udp,
                            internal_port_nonzero,
                            port_mapping_options
                        ).await?;
                        println!(
                            "UDP Port mapping added: External port {} -> {}:{}",
                            external_port, internal_ip, internal_port
                        );
                    }
                    _ => println!("Invalid protocol selection."),
                }

                println!("Mapping will be active for {} seconds", ttl);

            } else {
                println!("Gateway is not an IPv4 address");
            }
        }
        Err(e) => {
            eprintln!("Failed to get default gateway: {}", e);
        }
    }

    Ok(())
}