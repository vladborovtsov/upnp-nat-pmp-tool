use futures::prelude::*;
use rupnp::ssdp::{SearchTarget, URN};
use std::io::{self, Write}; // For user input and output
use std::time::Duration;

const WAN_IP_CONNECTION: URN = URN::service("schemas-upnp-org", "WANIPConnection", 1);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Search for the router (WANIPConnection service)
    let search_target = SearchTarget::URN(WAN_IP_CONNECTION);
    let devices = rupnp::discover(&search_target, Duration::from_secs(3)).await?;
    pin_utils::pin_mut!(devices);

    if let Some(device) = devices.try_next().await? {
        let service = device
            .find_service(&WAN_IP_CONNECTION)
            .expect("searched for WANIPConnection, got something else");

        // Fetch external IP by default
        println!("Found device: {}", device.friendly_name());
        let response = service.action(device.url(), "GetExternalIPAddress", "").await?;
        let external_ip = response.get("NewExternalIPAddress").map_or("Unknown", |v| v);

        println!("External IP Address: {}", external_ip);

        loop {
            // Present user with options
            println!("\nWhat would you like to do?");
            println!("1. List existing port mappings");
            println!("2. Add a new port mapping");
            println!("3. Exit");
            print!("Enter your choice: ");
            io::stdout().flush()?; // Make sure the prompt is printed immediately

            // Get user input
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim() {
                "1" => {
                    // List existing port mappings
                    println!("Listing port mappings...");
                    let mut index = 0;
                    loop {
                        let args = format!(
                            "<NewPortMappingIndex>{}</NewPortMappingIndex>",
                            index
                        );
                        match service.action(device.url(), "GetGenericPortMappingEntry", &args).await {
                            Ok(response) => {
                                let external_port = response.get("NewExternalPort").map_or("Unknown", |v| v);
                                let protocol = response.get("NewProtocol").map_or("Unknown", |v| v);
                                let internal_ip = response.get("NewInternalClient").map_or("Unknown", |v| v);
                                let internal_port = response.get("NewInternalPort").map_or("Unknown", |v| v);
                                let description = response.get("NewPortMappingDescription").map_or("Unknown", |v| v);

                                println!(
                                    "Mapping #{}: {}:{} -> {}:{} ({})",
                                    index, external_port, protocol, internal_ip, internal_port, description
                                );

                                index += 1;
                            }
                            Err(_) => {
                                println!("No more port mappings found.");
                                break;
                            }
                        }
                    }
                }
                "2" => {
                    // Add a new port mapping
                    println!("Enter internal IP: ");
                    let mut internal_ip = String::new();
                    io::stdin().read_line(&mut internal_ip)?;

                    println!("Enter internal port: ");
                    let mut internal_port = String::new();
                    io::stdin().read_line(&mut internal_port)?;

                    println!("Enter external port: ");
                    let mut external_port = String::new();
                    io::stdin().read_line(&mut external_port)?;

                    // Prompt for protocol option
                    println!("Select protocol:");
                    println!("1. TCP");
                    println!("2. UDP");
                    println!("3. BOTH");
                    print!("Enter your choice (1/2/3): ");
                    io::stdout().flush()?; // Make sure the prompt is printed immediately

                    let mut protocol_choice = String::new();
                    io::stdin().read_line(&mut protocol_choice)?;

                    match protocol_choice.trim() {
                        "1" => {
                            // Add a TCP port mapping
                            let args = format!(
                                "<NewRemoteHost></NewRemoteHost>
                                 <NewExternalPort>{}</NewExternalPort>
                                 <NewProtocol>TCP</NewProtocol>
                                 <NewInternalPort>{}</NewInternalPort>
                                 <NewInternalClient>{}</NewInternalClient>
                                 <NewEnabled>1</NewEnabled>
                                 <NewPortMappingDescription>InteractiveMapping</NewPortMappingDescription>
                                 <NewLeaseDuration>0</NewLeaseDuration>",
                                external_port.trim(),
                                internal_port.trim(),
                                internal_ip.trim()
                            );
                            let response = service.action(device.url(), "AddPortMapping", &args).await?;
                            println!("TCP Port mapping added: {:?}", response);
                        }
                        "2" => {
                            // Add a UDP port mapping
                            let args = format!(
                                "<NewRemoteHost></NewRemoteHost>
                                 <NewExternalPort>{}</NewExternalPort>
                                 <NewProtocol>UDP</NewProtocol>
                                 <NewInternalPort>{}</NewInternalPort>
                                 <NewInternalClient>{}</NewInternalClient>
                                 <NewEnabled>1</NewEnabled>
                                 <NewPortMappingDescription>InteractiveMapping</NewPortMappingDescription>
                                 <NewLeaseDuration>0</NewLeaseDuration>",
                                external_port.trim(),
                                internal_port.trim(),
                                internal_ip.trim()
                            );
                            let response = service.action(device.url(), "AddPortMapping", &args).await?;
                            println!("UDP Port mapping added: {:?}", response);
                        }
                        "3" => {
                            // Add both TCP and UDP port mappings
                            let args_tcp = format!(
                                "<NewRemoteHost></NewRemoteHost>
                                 <NewExternalPort>{}</NewExternalPort>
                                 <NewProtocol>TCP</NewProtocol>
                                 <NewInternalPort>{}</NewInternalPort>
                                 <NewInternalClient>{}</NewInternalClient>
                                 <NewEnabled>1</NewEnabled>
                                 <NewPortMappingDescription>InteractiveMapping</NewPortMappingDescription>
                                 <NewLeaseDuration>0</NewLeaseDuration>",
                                external_port.trim(),
                                internal_port.trim(),
                                internal_ip.trim()
                            );
                            let args_udp = format!(
                                "<NewRemoteHost></NewRemoteHost>
                                 <NewExternalPort>{}</NewExternalPort>
                                 <NewProtocol>UDP</NewProtocol>
                                 <NewInternalPort>{}</NewInternalPort>
                                 <NewInternalClient>{}</NewInternalClient>
                                 <NewEnabled>1</NewEnabled>
                                 <NewPortMappingDescription>InteractiveMapping</NewPortMappingDescription>
                                 <NewLeaseDuration>0</NewLeaseDuration>",
                                external_port.trim(),
                                internal_port.trim(),
                                internal_ip.trim()
                            );

                            let response_tcp = service.action(device.url(), "AddPortMapping", &args_tcp).await?;
                            let response_udp = service.action(device.url(), "AddPortMapping", &args_udp).await?;
                            println!("TCP and UDP Port mappings added: {:?}, {:?}", response_tcp, response_udp);
                        }
                        _ => println!("Invalid protocol choice, please try again."),
                    }
                }
                "3" => {
                    // Exit the loop and end the program
                    println!("Exiting...");
                    break;
                }
                _ => {
                    println!("Invalid option, please try again.");
                }
            }
        }
    } else {
        println!("No UPnP device found.");
    }

    Ok(())
}