use laminar::{Socket, Packet, SocketEvent};
use bincode::{deserialize, serialize};
use udp_controllers::*;
use keyboard_query::{DeviceQuery, DeviceState};
use std::net::{SocketAddr, SocketAddrV4, IpAddr, Ipv4Addr};
use std::time::{Duration};
use std::thread;
use anyhow::Result;

fn main() -> Result<(), anyhow::Error> {
    let local_ip = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 21032);
    let destination = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 43243);

    let mut client = Socket::bind(local_ip)?;

    let (packet_sender, packet_receiver) = (client.get_packet_sender(), client.get_event_receiver());
    
    let _thread = thread::spawn(move || client.start_polling());

    let message = Packet::reliable_unordered(
        destination,
        bincode::serialize(&FromClientMessage::Connect)?,
    );
    
    packet_sender.send(message)?;

    while let Ok(packet) = packet_receiver.recv() {
        match packet {
            SocketEvent::Packet(packet) => {
                println!("Connected to server: {:?}", packet.addr());
                break;
            },
            _ => {},
        }
    }
    
    let device_state = DeviceState::new();
    let mut prev_keys = vec![];
    loop {
        let keys = device_state.get_keys();
        if keys != prev_keys {
            prev_keys = keys;
            println!("{:?}", prev_keys);
            let packet = Packet::unreliable(
                destination,
                bincode::serialize(&prev_keys)?,
            );
            packet_sender.send(packet)?;
        } else {
            let packet = Packet::unreliable(
                destination,
                bincode::serialize(&prev_keys)?,
            );
            packet_sender.send(packet)?;
        }
    }
    
    Ok(())

}
