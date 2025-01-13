use etherparse::IpNumber;
use tappers::macos::Utun;
use std::net::Ipv4Addr;
use std::{collections::HashMap, io};
use tappers::{AddAddressV4, Tun};
type Port = u16;
mod tcp;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Quad {
    src: (Ipv4Addr, Port),
    dst: (Ipv4Addr, Port),
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tun = Utun::new()?;
    let mut addr = AddAddressV4::new(Ipv4Addr::new(10, 100, 0, 1));
    addr.set_destination(Ipv4Addr::new(10, 100, 0, 2));
    tun.add_addr(addr)?;
    tun.set_state(tappers::DeviceState::Up)?;
    // tun.set_up()?; // Enables the TUN device to exchange packets

    let mut connections: HashMap<Quad, tcp::Connection> = Default::default();
    let mut recv_buf = [0; 1500];

    loop {
        //this lib's received packet is not in the same format as the original packet
        //it doesn't contain the packet info (4 bytes)
        let length = tun.recv(&mut recv_buf)?;

        match etherparse::Ipv4HeaderSlice::from_slice(&recv_buf[0..length]) {
            Ok(ip_header) => {
                let src = ip_header.source_addr();
                let dst = ip_header.destination_addr();
                let protocol = ip_header.protocol();
                if protocol != IpNumber::TCP {
                    continue;
                }
                println!("IP packet received");
                match etherparse::TcpHeaderSlice::from_slice(
                    &recv_buf[ip_header.slice().len()..length],
                ) {
                    Ok(tcp_header) => {
                        use std::collections::hash_map::Entry;
                        println!("TCP packet received");
                        let data_index = ip_header.slice().len() + tcp_header.slice().len();
                        match connections.entry(Quad {
                            src: (src, tcp_header.source_port()),
                            dst: (dst, tcp_header.destination_port()),
                        }) {
                            Entry::Vacant(entry) => {
                                match tcp::Connection::accept(
                                    &mut tun,
                                    ip_header,
                                    tcp_header,
                                    &recv_buf[data_index..length],
                                ) {
                                    Ok(None) => (),
                                    Ok(Some(conn)) => {entry.insert(conn);},
                                    Err(e) => println!("error: {:?}", e )
                                }
                            }
                            Entry::Occupied(mut entry) => {
                                entry.get_mut().on_packet(
                                    &mut tun,
                                    ip_header,
                                    tcp_header,
                                    &recv_buf[data_index..length],
                                )?;
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error parsing TCP header: {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("Error parsing IPv4 header: {:?}", e);
            }
        }
    }
}
