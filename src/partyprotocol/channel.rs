use std::{
    fmt,
    net::{SocketAddr, SocketAddrV4, UdpSocket},
    sync::Arc,
    thread,
};
use super::packet::Packet;
use prost::Message;

pub struct ChannelConfig {
    name: String,
    handler: Cb,
}

type Cb = Box<dyn Fn(Packet, SocketAddr)->() + Send + 'static>;


struct ReceiveChannel{
    name: String,
    id: u32,
    callback: Cb,
}

impl ReceiveChannel{
    pub fn new(name: String, id: u32, callback: Cb)->ReceiveChannel{
        ReceiveChannel{
            name: name,
            id,
            callback,
        }
    }
}

pub struct Channel {
    name: String,
    id: u32,
    socket: Arc<UdpSocket>,
    callback: Option<Cb>,
}

impl Channel {
    pub fn new(name: String, socket: &Arc<UdpSocket>, id: u32, handler: Cb) -> Channel {
        Channel {
            name,
            id,
            socket: socket.clone(),
            callback: Some(handler),
        }
    }

    /// Builds a Vec of channels, and start listening at addr.
    ///
    /// # Arguments
    /// - addr: The address to listen at/send to. Should be a multicast ip:port
    /// - channel_names: A vector of strings. Will be used to build channels. Channel IDs are determined based on the order of appearance.
    ///
    /// # Returns Vec<Channel>
    pub fn build(addr: Option<&str>, channel_configs: Vec<ChannelConfig>) -> Vec<Channel> {
        let addr: SocketAddrV4 = addr
            .unwrap_or("239.195.10.10:8355")
            .parse()
            .expect("Cannot parse provided str as socket address");

        let bind_addr = SocketAddrV4::new("0.0.0.0".parse().unwrap(), addr.port());
        let socket = UdpSocket::bind(bind_addr)
            .expect("Cannot bind address. Isanother instance already running?");
        let socket = Arc::new(socket);

        let mut channels: Vec<_> = channel_configs
            .into_iter()
            .enumerate()
            .map(|(id, cfg)|Channel::new(cfg.name, &socket, id as u32, cfg.handler)
            )
            .collect();

        let receive_channels: Vec<_>=channels
            .iter_mut()
            .map(|ch|ReceiveChannel::new(
                ch.name.clone(),
                ch.id,
                ch.callback.take().unwrap()
            ))
            .collect();

        thread::spawn(move || loop {
            const BUF_LENGTH: usize = 4096;
            let mut buf = [0; BUF_LENGTH];
            let result = socket.recv_from(&mut buf);

            let Ok((length,src)) = result else{
                println!("Error occurred when receiving data from UDP socket");
                continue;
            };

            if length >= BUF_LENGTH {
                println!("Probable truncation occurred when receiving data from UDP socket");
            }

            let valid_buf=&buf[..length];
            let packet=Packet::decode(valid_buf);

            let Ok(packet)=packet else{
                println!("Received an invalid packet");
                continue;
            };
            if packet.version!=1 {
                println!("Received an unsupported packet: version incompatible");
                continue;
            }

            for ch in &receive_channels{
                if ch.id==packet.channel_id{
                    (ch.callback)(packet.clone(),src);

                    // Channel id should be unique
                    break;
                }
            }
        });
        channels
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Channel {}", self.name)
    }
}

impl fmt::Display for ReceiveChannel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ReceiveChannel {}", self.name)
    }
}
