use std::{
    fmt,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
    ops::Deref,
    rc::Rc,
    thread, vec,
};

pub struct ChannelConfig {
    name: String,
    handler: Box<dyn Fn() -> ()>,
}

pub struct Channel {
    name: String,
    id: i32,
    socket: Rc<UdpSocket>,
    callback: Box<dyn Fn() -> ()>,
}

impl Channel {
    pub fn new(name: String, socket: &Rc<UdpSocket>, id: i32, handler: impl Fn() -> ()) -> Channel {
        Channel {
            name,
            id,
            socket: socket.clone(),
            callback: Box::new(handler),
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
        let socket = Rc::new(socket);

        let channels = channel_configs
            .into_iter()
            .enumerate()
            .map(|(id, cfg)| Channel::new(cfg.name, &socket, id as i32, cfg.handler))
            .collect();

        thread::spawn(move || loop {
            const buf_length: usize = 4096;
            let mut buf = [0; buf_length];
            let result = socket.recv_from(&mut buf);

            let Ok((length,src)) = result else{
                println!("Error occurred when receiving data from UDP socket");
                continue;
            };

            if length == buf_length {
                println!("Probable truncation occurred when receiving data from UDP socket");
            }

            for channel in channels {}
        });

        channels
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
