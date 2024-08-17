use std::{char::MAX, collections::HashMap, sync::Arc, time::Duration};
use anyhow::{Result, ensure};
use tokio::{net::UdpSocket, select, sync::mpsc, time};
use bytes::{Buf, Bytes, BufMut, BytesMut};
use clap::{Parser, Command};
use std::net::{IpAddr, SocketAddr};
use rkyv::{ser::serializers::AllocSerializer, Archive, Deserialize, Serialize};
use vigem_client::Client;

pub mod key_mapper;

use crate::key_mapper::{KeyMapper, UserInput, ClientMessage};

const MAX_PAYLOAD: usize = 65507;
const DEFAULT_SERVER_PORT: u16 = 45681;
const DEFAULT_CLIENT_PORT: u16 = 45682;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    relay: bool,
    #[arg(long)]
    server: bool,
    relay_addr: Option<String>,
    addr: Option<String>,
    port: Option<u16>,
    config: Option<Path>
}

#[derive(Archive, Deserialize, Serialize, Debug)]
struct Handshake {
    role: u8,
    addr: SocketAddr,
}

#[derive(Archive, Deserialize, Serialize, Debug)]
struct Keys {
    keys: Vec<u8>
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum MessageType {
    Host = 0,
    Client = 1,
    Other,
}

impl From<u8> for MessageType {
    fn from(num: u8) -> Self {
        match num {
            0 => MessageType::Host,
            1 => MessageType::Client,
            _ => MessageType::Other
        }
    }
}

async fn relay(addr: Option<String>, port: u16) -> Result<()> {
    let conn = UdpSocket::bind(format!("{}:{port}", addr.unwrap_or("0.0.0.0".to_string()))).await?;
    
    let mut host: Option<SocketAddr> = None;
    let mut clients: Vec<SocketAddr> = Vec::new();

    loop {
        let mut buffer = [0; 1];
        let (_, addr) = conn.recv_from(&mut buffer).await?;
        let role = u8::from_be_bytes(buffer);
        match MessageType::from(role) {
            MessageType::Host => {
                // If there's already a host, do nothing
                if host.is_none() {
                    let host_addr = addr;
                    // Send host info to each client
                    for client in clients.iter() {
                        let host_info = &rkyv::to_bytes::<_, MAX_PAYLOAD>(&host_addr).expect("Failed to serialize host info");
                        conn.send_to(host_info, client).await?;
                    }
                    
                    // Send list of clients to host address to punch through to
                    let clients_msg = &rkyv::to_bytes::<_, MAX_PAYLOAD>(&clients).expect("Failed to serialize list of clients");
                    conn.send_to(clients_msg, addr);
                    host = Some(addr);
                }
            },
            MessageType::Client => {
                // Exchange host and client information
                if let Some(host_addr) = host {
                    let host_info = &rkyv::to_bytes::<_, MAX_PAYLOAD>(&host_addr).expect("Failed to serialize host info");
                    let client_info = &rkyv::to_bytes::<_, MAX_PAYLOAD>(&addr).expect("Failed to serialize client info");
                    conn.send_to(host_info, addr).await?;
                    conn.send_to(client_info, host_addr).await?;
                }
                clients.push(addr);
            }
            _ => {}
        }

    } 
}

async fn setup_client(client: &SocketAddr, vigem: Arc<Client>) -> mpsc::Sender<ClientMessage> {
    let (tx, mut rx) = mpsc::channel::<ClientMessage>(1000);
    let mut controller = vigem_client::Xbox360Wired::new(vigem, vigem_client::TargetId::XBOX360_WIRED);
    controller.plugin().expect("Failed to plugin controller");
    controller.wait_ready().expect("Failed to wait ready controller");

    tokio::spawn(async move {
        loop {
            match time::timeout(Duration::from_secs(10), rx.recv()).await {
                Ok(Some(message)) => {
                    match message {
                        ClientMessage::Input(input) => {
                            let gamepad = vigem_client::Xgamepad {
                                thumb_lx: input.lx,
                                thumb_ly: input.ly,
                                thumb_rx: input.rx,
                                thumb_ry: input.ry,
                                left_trigger: input.ltrigger,
                                right_trigger: input.rtrigger,
                                buttons: input.buttons
                            };
                        
                            if let Err(e) = controller.update(&gamepad) {
                                eprintln!("Error updating controller: {:?}", e);
                            }
                        }
                        ClientMessage::Hearbeat => {},
                    }
                },
                _ => {
                    // Close controller
                    controller.unplug();
                    return;
                }
            }
        }
    });

    return tx;
}

async fn server(relay_addr: SocketAddr, server_addr: Option<String>, server_port: Option<u16>) -> Result<()> {
    let conn = Arc::new(UdpSocket::bind(format!("{}:{}", server_addr.unwrap_or("0.0.0.0".to_string()), server_port.unwrap_or(DEFAULT_SERVER_PORT))).await?);
    let mut client_channels: HashMap<SocketAddr, mpsc::Sender<ClientMessage>> = HashMap::new();

    // UDP Punchthrough
    let role: [u8; 1] = [0];
    conn.send_to(&role, relay_addr);

    let mut clients_buffer = [0; MAX_PAYLOAD];
    let bytes_recv: usize;
    let recv_addr: SocketAddr;
    match time::timeout(Duration::from_secs(5), conn.recv_from(&mut clients_buffer)).await {
        Ok((bytes_recv, recv_addr)) => {
            bytes_recv = bytes_recv;
            recv_addr = recv_addr; 
        }
        Err(_) => { panic!("Failed to connect to the relay server") }
    }
    /*
    if let Err(_) = time::timeout(Duration::from_secs(5), conn.recv_from(&mut clients_buffer)).await {
        panic!("Failed to connect to the relay server");
    }
    */
    let vigem: Arc<Client> = Arc::new(vigem_client::Client::connect().expect("Can't retrieve vigem client: 159"));

    let archived_clients = unsafe { rkyv::archived_root::<Vec<SocketAddr>>(&clients_buffer[..bytes_recv]) };
    let clients: Vec<SocketAddr> = archived_clients.deserialize(&mut rkyv::Infallible).unwrap();
    for client in clients {
        let tx = setup_client(&client, vigem.clone()).await;
        client_channels.insert(client, tx);
    }


    loop {
        let mut buffer = [0; MAX_PAYLOAD];
        let (bytes_recv, addr) = conn.recv_from(&mut buffer).await?;

        if let Some(channel) = client_channels.get(&addr) {
            // Message from existing client
            let archived_message = unsafe { rkyv::archived_root::<ClientMessage>(&buffer[..bytes_recv]) };
            let client_message: ClientMessage = archived_message.deserialize(&mut rkyv::Infallible).unwrap();
            channel.send(client_message);
        } else if addr == relay_addr {
            // Message from relay server for new clients
            let archived_clients = unsafe { rkyv::archived_root::<Vec<SocketAddr>>(&buffer[..bytes_recv]) };
            let clients: Vec<SocketAddr> = archived_clients.deserialize(&mut rkyv::Infallible).unwrap();
            for client in clients {
                let tx = setup_client(&client, vigem.clone()).await;
                client_channels.insert(client, tx);
            }
        } else {
            // Message from client we haven't connected to yet
        }
    }
}

async fn client(relay_addr: SocketAddr, client_addr: Option<String>, client_port: Option<u16>, key_mapper: KeyMapper) -> Result<()> {
    let conn = UdpSocket::bind(format!("{}:{}", client_addr.unwrap_or("0.0.0.0".to_string()), client_port.unwrap_or(DEFAULT_CLIENT_PORT))).await?;

    // UDP Punchthrough 
    let role: [u8; 1] = [1];
    conn.send_to(&role, relay_addr);

    let mut host_buffer = [0; MAX_PAYLOAD];
    let bytes_recv: usize;
    let recv_addr: SocketAddr;
    match time::timeout(Duration::from_secs(5), conn.recv_from(&mut host_buffer)).await {
        Ok((bytes_recv, recv_addr)) => {
            bytes_recv = bytes_recv;
            recv_addr = recv_addr;
        }
        Err(_) => { panic!("Failed to connect to the relay server"); }
    }
    /* 
    if let Err(_) = time::timeout(Duration::from_secs(5), conn.recv_from(&mut host_buffer)).await {
        panic!("Failed to connect to the relay server");
    }
    */

    let archived_host = unsafe { rkyv::archived_root::<SocketAddr>(&host_buffer[..bytes_recv]) };
    let host: SocketAddr = archived_host.deserialize(&mut rkyv::Infallible).unwrap();

    // TODO! Figure out the juggling between querying keys and the heartbeat timer
    let mut prev_input: Option<UserInput> = None;
    let mut query = time::interval(Duration::from_millis(5));
    let mut heartbeat = time::interval(Duration::from_secs(5));
    heartbeat.tick().await;
    query.tick().await;

    loop {
        //https://stackoverflow.com/questions/68961504/non-blocking-recv-on-tokio-mpsc-receiver
        tokio::select! {
            _ = heartbeat.tick() => {
                // Send heartbeat message
                let hearbeat_bytes = &rkyv::to_bytes::<_, MAX_PAYLOAD>(&ClientMessage::Hearbeat).expect("Failed to serialize hearbeat message");
                conn.send_to(hearbeat_bytes, host).await?;
            }
            _ = query.tick() => {
                let input = key_mapper.get_input()?;
                if let Some(last) = prev_input {
                    if last == input {
                        continue;
                    }
                }
                prev_input = Some(input);

                let input_bytes = &rkyv::to_bytes::<_, MAX_PAYLOAD>(&ClientMessage::Input(input)).expect("Failed to serialize user input");
                conn.send_to(input_bytes, host).await?;
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();    

    if args.relay {
        ensure!(args.port.is_some(), "The port needs to be set if running as a relay");
        run_relay().await?;
    } 

    if args.server {
        ensure!(args.relay_addr.is_some(), "A relay address needs to be provided");
        server(args.relay_addr.unwrap(), args.addr, args.port).await?;
    } else {
        ensure!(args.relay_addr.is_some(), "A relay address needs to be provided");
        ensure!(args.config.is_some(), "A controller config path needs to be provided");
        let keymap = KeyMapper::new(args.config.unwrap())?;
        client(args.relay_addr.unwrap(), args.addr, args.port, keymap).await?;
    }

    Ok(())
    // Connect to relay server for UDP hole punch

    // If server, start listening and controller manager
    //https://docs.rs/tokio/1.39.2/tokio/net/struct.UdpSocket.html <- Socket splitting with Arc, spawn new tasks for each new connection that listens for heartbeats and manages controllers, communicate with channels

    // else client, start recording keybinds and sending to host
}
