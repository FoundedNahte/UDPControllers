use laminar::{Socket, Packet, SocketEvent};
//use bincode::{deserialize, serialize};
//use udp_controllers::FromClientMessage;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::time::Duration;
use std::thread;
use std::collections::HashMap;
use anyhow::Result;

fn main() -> Result<(), anyhow::Error> {
    let local_ip = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8888);
    
    let mut server = Socket::bind(local_ip)?; 
    let (packet_sender, packet_receiver) = (server.get_packet_sender(), server.get_event_receiver());
    
    let _thread = thread::spawn(move || server.start_polling());
    
    let mut connections: HashMap<SocketAddr, (vigem_client::Xbox360Wired::<vigem_client::Client>, vigem_client::XGamepad)> = HashMap::new();
    
    loop {
        match packet_receiver.recv() {
            Ok(socket_event) => {
                (|| { match socket_event {
                    SocketEvent::Packet(packet) => {
                        let (target, gamepad) = match connections.get_mut(&packet.addr()) {
                            Some((target, gamepad)) => {
                                (target, gamepad)
                            },
                            None => {
                                println!("No connection present for {:?}", packet.addr());
                                let message = Packet::reliable_unordered(
                                    packet.addr(),
                                    bincode::serialize(&0).expect("Error serializing connection message"),
                                );
                                
                                match packet_sender.send(message) {
                                    Ok(()) => {},
                                    Err(e) => {
                                        eprintln!("Failed to return handshake: {:?}", e);
                                    }
                                } 

                                return 
                            }
                        };
                        
                        let keys: Vec<u16> = match bincode::deserialize(&packet.payload()) {
                            Ok(command) => { command },
                            Err(e) => {
                                eprintln!("Error deserializing payload: {:?}", e);
                                return;
                            }
                        };
                       
                        println!("{:?}", keys);

                        gamepad.right_trigger = 0;
                        gamepad.left_trigger = 0;
                        gamepad.thumb_ly = 0;
                        gamepad.thumb_lx = 0;
                        gamepad.buttons = vigem_client::XButtons(0);

                        for key in keys.iter() {
                            match key {
                                // W Left Joystick Up
                                87 => { gamepad.thumb_ly = 29999; },
                                // A Left Joystick Left
                                65 => { gamepad.thumb_lx = -29999; },
                                // D Left Joystick Right
                                68 => { gamepad.thumb_lx = 29999; },
                                // S Left Joystick Down
                                83 => { gamepad.thumb_ly = -29999 },
                                // Up-Arrow Y Button
                                38 => { gamepad.buttons = vigem_client::XButtons(0x8000); },
                                // Left-Arrow X Button
                                37 => { gamepad.buttons = vigem_client::XButtons(0x4000); },
                                // Right-Arrow B Button
                                39 => { gamepad.buttons = vigem_client::XButtons(0x2000); },
                                // Down-Arrow A Button
                                40 => { gamepad.buttons = vigem_client::XButtons(0x1000); },
                                // Q Left Bumper
                                81 => { gamepad.buttons = vigem_client::XButtons(0x0100); },
                                // E Right Bumper
                                69 => { gamepad.buttons = vigem_client::XButtons(0x0200); },
                                // P Start Button
                                80 => { gamepad.buttons = vigem_client::XButtons(0x0010); },
                                // O Back Button
                                79 => { gamepad.buttons = vigem_client::XButtons(0x0020); },
                                // U Right Trigger
                                85 => { gamepad.right_trigger = 255; },
                                // U Left Trigger
                                89 => { gamepad.left_trigger = 255; },
                                // I D-Pad UP
                                73 => { gamepad.buttons = vigem_client::XButtons(0x0001); },
                                // J D-Pad Left
                                74 => { gamepad.buttons = vigem_client::XButtons(0x0004); },
                                // L D-Pad Right
                                76 => { gamepad.buttons = vigem_client::XButtons(0x0008); },
                                // K D-Pad Down
                                75 => { gamepad.buttons = vigem_client::XButtons(0x0002); },
                                _ => {},
                            }    
                        }
                        
                        match target.update(&gamepad) {
                            Ok(()) => {},
                            Err(e) => {
                                eprintln!("Error updating controller: {:?}", e);
                            }
                        }

                    },
                    SocketEvent::Connect(addr) => {
                        match connections.get(&addr) {
                            Some(_connection) => {},
                            None => {
                                let client = vigem_client::Client::connect().expect("Can't retrieve vigem client: 159");

                                let id = vigem_client::TargetId::XBOX360_WIRED;
                                let mut target = vigem_client::Xbox360Wired::new(client, id);

                                target.plugin().expect("Error pluging in target: 164");

                                target.wait_ready().expect("Error: 166");

                                let gamepad = vigem_client::XGamepad {
                                    buttons: vigem_client::XButtons!(A | B | X | Y | LB | RB | START | BACK | UP | DOWN | RIGHT | LEFT),
                                    ..Default::default()
                                };

                                match connections.insert( addr, (target, gamepad)) {
                                    Some(_connection) => {},
                                    None => {
                                        println!("Successfully registered connection: {:?}", addr);
                                    }
                                }
                            },
                        }
                    },
                    SocketEvent::Disconnect(addr) => {},
                    _ => {},
                }})();
            },
            Err(e) => {
                eprintln!("Something went wrong when receiving, error: {:?}", e);
            }
        }
    }
}