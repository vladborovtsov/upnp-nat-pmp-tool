# upnp-nat-pmp-tool

Small Rust utilities to inspect and manage home router port mappings via:
- UPnP IGD (Internet Gateway Device) `WANIPConnection`
- NAT-PMP (NAT Port Mapping Protocol)

This repository provides two binaries:
- `upnp-tool` — discover a UPnP-capable router, show the external IP, list existing port mappings, and interactively add a new mapping (TCP/UDP/BOTH)
- `nat-pmp-tool` — use NAT-PMP to fetch the external address and add a temporary port mapping (TCP/UDP/BOTH) with a configurable TTL

License: MIT (see `LICENSE`).


## Features and binaries

Cargo features gate the two binaries and their dependencies:

- Feature `upnp` enables the `upnp-tool` binary (uses `rupnp`, `pin-utils`).
- Feature `pmp` enables the `nat-pmp-tool` binary (uses `crab_nat`, `default-net`).

From `Cargo.toml`:

```toml
[features]
upnp = ["pin-utils", "rupnp"]
pmp  = ["crab_nat", "default-net"]

[[bin]]
name = "upnp-tool"
path = "src/upnp-tool/main.rs"
required-features = ["upnp"]

[[bin]]
name = "nat-pmp-tool"
path = "src/nat-pmp-tool/main.rs"
required-features = ["pmp"]
```


## Requirements

- Rust 1.70+ recommended (edition 2021; Tokio 1.x)
- A router that supports either:
  - UPnP IGD (for `upnp-tool`), or
  - NAT-PMP (for `nat-pmp-tool`)
- Your machine must be on the same network as the router and able to reach it (local firewalls may need to allow discovery/UDP traffic).

Platform notes:
- Linux/macOS/Windows should all work. On some systems, you may need to run from a terminal with adequate permissions if local firewall rules are restrictive.


## Build

Build each binary with its feature:

```bash
# Build the UPnP tool
cargo build --release --features upnp --bin upnp-tool

# Build the NAT-PMP tool
cargo build --release --features pmp --bin nat-pmp-tool
```

This produces the executables under `target/release/`:
- `target/release/upnp-tool`
- `target/release/nat-pmp-tool`

You can also run them without a release build during development:

```bash
cargo run --features upnp --bin upnp-tool
cargo run --features pmp  --bin nat-pmp-tool
```


## Usage

### upnp-tool (UPnP IGD)

`upnp-tool` discovers a router that exposes the UPnP `WANIPConnection` service, prints your external IP address, and provides a simple interactive menu:

- List existing port mappings
- Add a new port mapping (TCP/UDP/BOTH)

Example session:

```text
$ cargo run --features upnp --bin upnp-tool
Found device: My Home Router
External IP Address: 203.0.113.25

What would you like to do?
1. List existing port mappings
2. Add a new port mapping
3. Exit
Enter your choice: 1
Listing port mappings...
Mapping #0: 443:TCP -> 192.168.1.10:443 (HTTPS)
Mapping #1: 51820:UDP -> 192.168.1.20:51820 (WireGuard)
No more port mappings found.

What would you like to do?
1. List existing port mappings
2. Add a new port mapping
3. Exit
Enter your choice: 2
Enter internal IP: 192.168.1.42
Enter internal port: 8080
Enter external port: 8080
Select protocol:
1. TCP
2. UDP
3. BOTH
Enter your choice (1/2/3): 3
TCP and UDP Port mappings added: ...
```

Notes:
- New UPnP mappings in this tool use `NewLeaseDuration=0` (often interpreted as permanent on many routers). You may need to remove them manually via your router UI or another tool.
- Listing stops when the router stops returning entries.


### nat-pmp-tool (NAT-PMP)

`nat-pmp-tool` locates your default gateway, requests the external IP via NAT-PMP, then interactively creates a port mapping with a configurable lifetime (TTL in seconds). It also auto-detects a suitable internal IPv4 address on the same network as your gateway.

Example session:

```text
$ cargo run --features pmp --bin nat-pmp-tool
Default Gateway as Ipv4Addr: 192.168.1.1
External IP Address: 203.0.113.25
Found internal IP: 192.168.1.42
Enter internal port (default 8080): 8080
Enter external port (default 8080): 8080
Select protocol:
1. TCP
2. UDP
3. BOTH
Enter your choice (1/2/3): 1
TCP Port mapping added: External port 8080 -> 192.168.1.42:8080
Mapping will be active for 3600 seconds
```

Notes:
- NAT-PMP mappings in this tool are temporary; TTL defaults to 3600 seconds. Renew if you need longer.
- Some routers may not support NAT-PMP but may support UPnP IGD (or PCP). Use the matching tool for your router.


## Security considerations

- Opening port mappings exposes services on your LAN to the public Internet. Only forward ports you understand and keep your services updated.
- Prefer non-privileged ports and minimize exposure time (use NAT-PMP with short TTLs if possible).
- UPnP permanent mappings (lease `0`) can persist until you remove them.
- Ensure your firewall rules match what you intend to expose.


## Troubleshooting

- "No UPnP device found": ensure UPnP IGD is enabled on your router and that you are on the same network segment. Local firewalls may block SSDP multicast (239.255.255.250:1900).
- NAT-PMP failures: your router may not implement NAT-PMP (common on some ISP devices). Try `upnp-tool` instead if UPnP is available.
- "Gateway is not an IPv4 address": dual-stack environments might select IPv6 gateways; this tool expects IPv4 for NAT-PMP operations.
- Permission/firewall issues: run from a terminal, ensure outbound UDP and multicast are allowed on your machine.


## Development

The code uses:
- `tokio` and `futures` for async runtime and futures utilities
- `rupnp` for UPnP discovery and actions
- `crab_nat` for NAT-PMP operations
- `default-net` to query the default gateway and network interfaces

Minimal examples to run directly:

```bash
# UPnP
cargo run --features upnp --bin upnp-tool

# NAT-PMP
cargo run --features pmp --bin nat-pmp-tool
```


## Roadmap / ideas

- Optional delete/remove mapping commands
- PCP (Port Control Protocol) support where available
- Better diagnostics and structured output (e.g., JSON mode)


## License

see [LICENSE](./LICENSE)


## Acknowledgements

- [`rupnp`](https://crates.io/crates/rupnp)
- [`crab_nat`](https://crates.io/crates/crab_nat)
- [`default-net`](https://crates.io/crates/default-net)
- [`tokio`](https://crates.io/crates/tokio)
- [`futures`](https://crates.io/crates/futures)
