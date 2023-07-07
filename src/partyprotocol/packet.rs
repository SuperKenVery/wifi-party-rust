
mod custom{

    use safe_transmute::{transmute_to_bytes, transmute_to_bytes_vec, transmute_vec};

    use super::packet::Packet as ProtoPacket;
    pub struct Packet{
        pub version: u32,
        pub channel_id: u32,
        pub index: u32,
        pub data: Box<[i16]>,
    }

    impl Packet {
        fn encode(mut self, buf: &mut Vec<u8>)  {
            let data=transmute_to_bytes(&*self.data);

            let pp=ProtoPacket{
                version: self.version,
                channel_id: self.channel_id,
                index: self.index,
                data: *data.ve,
            };
            pp.encode(buf);

            self=pp.to_custom_packet().unwrap();
        }
    }
}

mod packet{
    use super::custom;
    use safe_transmute::{transmute_to_bytes_vec,transmute_many, SingleManyGuard, transmute_vec};


    include!(concat!(env!("OUT_DIR"), "/wifipartyrs.items.rs"));
    impl Packet{
        pub fn to_custom_packet(self) -> Result<Box<custom::Packet>,String>{
            let Ok(data)=transmute_vec::<u8,i16>(self.data) else{
                println!("#{} invalid: Failed to convert to i16",self.index);
                return;
            };

            transmute_vec(vec)

            transmute_vec

            Ok(Box::new(custom::Packet{
                version: self.version,
                channel_id: self.channel_id,
                index: self.index,
                data: *data,
            }))
        }

    }
}

pub use custom::Packet as Packet;

