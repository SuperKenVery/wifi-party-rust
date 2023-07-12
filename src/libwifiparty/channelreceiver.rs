// Receive from a channel, reassembly the sound segments
// into a buffer, and handle out-of-order delivery and
// packet loss.

use crate::partyprotocol::channel::ChannelConfig;

use super::super::partyprotocol::{channel::Channel, packet::Packet};
use std::array;
use std::net::SocketAddr;
use std::sync::{Arc,Mutex};


const BUFFER_SEGMENTS:usize=10;

pub struct ChannelReceiverBuildConfig{
    addr: Option<String>,
    names: Vec<String>,
}

struct A{
    s: [i16],
}
pub struct ChannelReceiver{
    name: String,
    segments: [Option<Box<[i16]>>;BUFFER_SEGMENTS],
    ridx: usize, // Read index, points to an already read space i.e. To read, use segments[ridx+1]
    widx: usize, // Write index, points to next available space
    lidx: u32, // Last received index
    missed: Vec<usize>, // What packet indexes are missing
}

impl ChannelReceiver{
    fn new(name: &str) -> ChannelReceiver {
        ChannelReceiver{
            name: name.to_string(),
            segments: array::from_fn(|_| None),
            ridx: 0,
            widx: 0,
            lidx: 0,
            missed: vec![],
        }
    }

    pub fn build(config: ChannelReceiverBuildConfig) -> (Vec<Arc<Mutex<ChannelReceiver>>>,Vec<Channel>){
        let mut receivers=Vec::with_capacity(config.names.len());
        let channel_configs=config
            .names
            .iter()
            .map(|name|{
                let receiver=Arc::new(
                    Mutex::new(
                            ChannelReceiver::new(name)
                    )
                );
                receivers.push(receiver.clone());

                let r=receiver.clone();
                ChannelConfig{
                    name: name.clone(),
                    handler: Box::new(move |packet: Packet, addr: SocketAddr|{
                        r.lock().unwrap().receive_packet(packet, addr);
                    }),
                }
            })
            .collect();

        let channels=Channel::build(channel_configs,config.addr.as_deref());

        (receivers,channels)

    }

    // Receive a packet, and handle it.
    fn receive_packet(&mut self, packet: Packet, addr: SocketAddr){
        if packet.get_index()==self.lidx+1{
            // Packet is in order
            self.lidx+=1;

            if self.widx==self.ridx+1{
                // Buffer is full, drop the oldest segment
                // |8|9|10|1|2|3|4|5|6|7|
                //      ^r ^w
                self.ridx+=1;
                if self.ridx==self.segments.len(){
                    self.ridx=0;
                }

                // self.segments[self.widx]=packet.data;
                self.widx+=1;
                if self.widx==self.segments.len(){
                    self.widx=0;
                }

            }
        }
    }

    // Read sound
    // Fills up the buffer
    fn read(&mut self, buffer: &mut [i16]){

    }
}