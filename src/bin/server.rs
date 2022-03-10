use laminar::{Socket, Packet, SocketEvent};
//use bincode::{deserialize, serialize};
use udp_controllers::FromClientMessage;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::time::Duration;
use std::thread;
use std::collections::HashMap;
use anyhow::Result;

pub struct Config {
    target: vigem_client::Xbox360Wired<vigem_client::Client>,
    gamepad: vigem_client::XGamepad,
    pub thumb_x: i32,
    pub thumb_y: i32,
    pub trigger_l: i32,
    pub trigger_r: i32,
}

impl Config {
    pub fn new() -> Self {
        let thumb_x = 0;
        let thumb_y = 0;
        let trigger_l = 0;
        let trigger_r = 0;

        let client = vigem_client::Client::connect().unwrap();
        let id = vigem_client::TargetId::XBOX360_WIRED;
        let mut target = vigem_client::Xbox360Wired::new(client, id);
        let gamepad = vigem_client::XGamepad { 
            buttons: vigem_client::XButtons!( A | B | X | Y | LB | RB | START | BACK | UP | DOWN | RIGHT | LEFT),
            ..Default::default()
        };

        target.plugin().unwrap();
        target.wait_ready().unwrap();

        Self {
            target,
            gamepad,
            thumb_x,
            thumb_y,
            trigger_l,
            trigger_r,
        } 
    }
}

fn main() -> Result<(), anyhow::Error> {
    let local_ip = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8888);

    let mut server = Socket::bind(local_ip)?;
    let (packet_sender, packet_receiver) = (server.get_packet_sender(), server.get_event_receiver());

    let _thread = thread::spawn(move || server.start_polling());

    let mut connections: HashMap<SocketAddr, (vigem_client::Xbox360Wired::<vigem_client::Client>, vigem_client::XGamepad)> = HashMap::new();
    //let mut connections: HashMap<SocketAddr, Config> = HashMap::new();  

    loop {
        match packet_receiver.recv() {
            Ok(socket_event) => {
                // CLOSURE TO ALLOW RETURNING FROM MATCH ARMS
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
                                    bincode::serialize(&FromClientMessage::Connect).unwrap(),
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
                    /*
                    SocketEvent::Packet(packet) => {
                        let config = match connections.get_mut(&packet.addr()) {
                            Some(config) => {
                                config
                            },
                            None => {
                                println!("No connection present for {:?}", packet.addr());
                                let message = Packet::reliable_unordered(
                                    packet.addr(),
                                    bincode::serialize(&FromClientMessage::Connect).unwrap(),
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
                        let target = &mut config.target;
                        let mut gamepad = &mut config.gamepad;
                       */
                        let command: FromClientMessage = match bincode::deserialize(&packet.payload()) {
                            Ok(command) => { command },
                            Err(e) => { 
                                eprintln!("Error deserializing payload: {:?}", e);
                                return;
                            },
                        };

                        println!("{:?}", command); 
                        match command {
                            // LEFT THUMBSTICK MOTION
                            FromClientMessage::W => {
                                //if config.thumb_y == 29999 {
                                    //return;
                                //}
                                gamepad.thumb_ly = 29999;
                                //config.thumb_y = 29999;
                            },
                            FromClientMessage::A => {
                                //if config.thumb_x == -29999 {
                                    //return;
                                //}
                                gamepad.thumb_lx = -29999;
                                //config.thumb_x = -29999;
                            },
                            FromClientMessage::D => {
                                //if config.thumb_x == 29999 {
                                    //return;
                                //}
                                gamepad.thumb_lx = 29999;
                                //config.thumb_x = 29999;
                            },
                            FromClientMessage::S => {
                                //if config.thumb_y == -29999 {
                                    //return;
                                //}
                                gamepad.thumb_ly = -29999;
                                //config.thumb_y = -29999;
                            },
                            // A, B, X, Y BUTTONS
                            FromClientMessage::Up => {
                                gamepad.buttons = vigem_client::XButtons(0x8000);
                            },
                            FromClientMessage::Left => {
                                gamepad.buttons = vigem_client::XButtons(0x4000);
                            },
                            FromClientMessage::Right => {
                                gamepad.buttons = vigem_client::XButtons(0x2000);
                            },
                            FromClientMessage::Down => {
                                gamepad.buttons = vigem_client::XButtons(0x1000);
                            },
                            // LEFT AND RIGHT BUMPERS
                            FromClientMessage::Q => {
                                gamepad.buttons = vigem_client::XButtons(0x0100);
                            },
                            FromClientMessage::E => {
                                gamepad.buttons = vigem_client::XButtons(0x0200);
                            },
                            // START BUTTON
                            FromClientMessage::P => {
                                gamepad.buttons = vigem_client::XButtons(0x0010);
                            },
                            // BACK BUTTON
                            FromClientMessage::O => {
                                gamepad.buttons = vigem_client::XButtons(0x0020);
                            },
                            // RIGHT TRIGGER
                            FromClientMessage::U => {
                                gamepad.right_trigger = 255;
                            },
                            // LEFT TRIGGER
                            FromClientMessage::Y => {
                                gamepad.left_trigger = 255;
                            },
                            // D-PAD BUTTONS
                            FromClientMessage::I => {
                                gamepad.buttons = vigem_client::XButtons(0x0001);
                            },
                            FromClientMessage::J => {
                                gamepad.buttons = vigem_client::XButtons(0x0004);
                            },
                            FromClientMessage::L => {
                                gamepad.buttons = vigem_client::XButtons(0x0008);
                            },
                            FromClientMessage::K => {
                                gamepad.buttons = vigem_client::XButtons(0x0002);
                            },
                            FromClientMessage::None => {
                                gamepad.right_trigger = 0;
                                gamepad.left_trigger = 0;
                                gamepad.thumb_ly = 0;
                                gamepad.thumb_lx = 0;
                                gamepad.buttons = vigem_client::XButtons(0);
                                //config.thumb_x = 0;
                                //config.thumb_y = 0;
                            },
                            _ => {},
                        }
                        match target.update(&gamepad) {
                            Ok(()) => {},
                            Err(e) => {
                                eprintln!("Error updating controller: {:?}", e);
                            },
                        }

                    },
                    SocketEvent::Connect(addr) => {
                        match connections.get(&addr) {
                            Some(_connection) => {},
                            None => {
                                
                                let client = vigem_client::Client::connect().unwrap();

                                let id = vigem_client::TargetId::XBOX360_WIRED;
                                let mut target = vigem_client::Xbox360Wired::new(client, id);
                                
                                target.plugin().unwrap();

                                target.wait_ready().unwrap();

                                let gamepad = vigem_client::XGamepad {
                                    buttons: vigem_client::XButtons!(A | B | X | Y | LB | RB | START | BACK | UP | DOWN| RIGHT | LEFT),
                                    ..Default::default()
                                };
                                
                                
                                //let config = Config::new();

                                match connections.insert(addr, (target, gamepad)) {
                                    Some(_connection) => {},
                                    None => {
                                        println!("Successfully registered connection: {:?}", addr);
                                    }
                                }
                            },
                        }
                    },
                    SocketEvent::Disconnect(addr) => {
                       /* 
                        match connections.remove(&addr) {
                            Some((mut target, _gamepad)) => {
                                target.unplug().unwrap();
                            },
                            None => { println!("Address not registered: {:?}", addr); },
                        }
                        */
                    },
                    _ => {},
                }})();
            }
            Err(e) => {
                eprintln!("Something went wrong when receiving, error: {:?}", e);
            }
        }
    }
}
