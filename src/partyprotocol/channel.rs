use std::{
    fmt,
    net::{SocketAddr, SocketAddrV4, UdpSocket},
    sync::Arc,
    thread,
};
use super::packet::{Packet};
use prost::Message;
use safe_transmute::transmute_to_bytes_vec;

pub struct ChannelConfig {
    name: String,
    handler: Cb,
}

type Cb = Box<dyn FnMut(Packet, SocketAddr)->() + Send + 'static>;


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
    addr: SocketAddrV4,
    callback: Option<Cb>,
    send_index: u32,
}

impl Channel {
    pub fn new(name: String, socket: Arc<UdpSocket>, addr: SocketAddrV4, id: u32, handler: Cb) -> Channel {
        Channel {
            name,
            id,
            socket,
            addr,
            callback: Some(handler),
            send_index: 0,
        }
    }

    /// Builds a Vec of channels, and start listening at addr.
    ///
    /// # Arguments
    /// - addr: The address to listen at/send to. Should be a multicast ip:port
    /// - channel_names: A vector of strings. Will be used to build channels. Channel IDs are determined based on the order of appearance.
    ///
    /// # Returns Vec<Channel>
    pub fn build(channel_configs: Vec<ChannelConfig>, addr: Option<&str>) -> Vec<Channel> {
        let addr: SocketAddrV4 = addr
            .unwrap_or("239.195.10.10:8355")
            .parse()
            .expect("Cannot parse provided str as socket address");

        let bind_addr = SocketAddrV4::new("0.0.0.0".parse().unwrap(), addr.port());
        let socket = UdpSocket::bind(bind_addr)
            .expect("Cannot bind address. Is another instance already running?");
        let socket = Arc::new(socket);

        let mut channels: Vec<_> = channel_configs
            .into_iter()
            .enumerate()
            .map(|(id, cfg)|Channel::new(cfg.name, socket.clone(), addr.clone(),id as u32, cfg.handler)
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

    fn send(self: &mut Channel, sound: Vec<i16>) -> Result<(),&str>{
        let mut packet=Packet::default();

        packet.version=1;
        packet.channel_id=self.id;
        packet.index=self.send_index;
        self.send_index+=1;

        let Ok(data)=transmute_to_bytes_vec(sound) else{
            return Err("Failed to convert sound data to bytes");
        };
        packet.data=data;

        let mut buf=Vec::with_capacity(packet.encoded_len());
        packet.encode(&mut buf);

        self.socket.send_to(&buf, self.addr);
        Ok(())
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
