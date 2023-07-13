use std::error::Error;

use rkyv::{
    Archive,
    Serialize,
    Deserialize,
    ser::{
        serializers::{BufferSerializer, AllocSerializer},
        Serializer,
    }, vec::ArchivedVec, AlignedVec,
};

#[derive(Archive,Deserialize,Serialize,Debug,PartialEq)]
#[archive(check_bytes,compare(PartialEq))]
#[archive_attr(derive(Debug))]
pub struct Header{
    pub identifier: [char;4],
    pub channel: u32,
    pub index: u32,
}



#[derive(Archive,Serialize,Debug,PartialEq)]
#[archive(check_bytes,compare(PartialEq))]
#[archive_attr(derive(Debug))]
pub struct ConstructedPacket{
    pub header: Header,
    pub data: Vec<i16>,
}

pub struct OwnedArchivedPacket{
    buffer: Vec<u8>,
}

pub enum Packet{
    Constructed(ConstructedPacket),
    Received(OwnedArchivedPacket),
}

impl Packet{
    pub fn new(channel: u32, index: u32, data: Vec<i16>) -> Packet{
        Packet::Constructed(ConstructedPacket::new(channel,index,data))
    }

    pub fn encode(&self) -> Result<AlignedVec,Box<dyn Error>>{
        match self{
            Packet::Constructed(packet) => packet.encode(),
            Packet::Received(_) => {
                Err("Sending a received packet".into())
            },
        }
    }

    pub fn decode(buf: Vec<u8>) -> Result<Packet,Box<dyn Error>>{
        let p=OwnedArchivedPacket::decode(buf)?;

        Ok(Packet::Received(p))
    }

    pub fn get_channel(&self) -> u32{
        match self{
            Packet::Constructed(packet) => packet.header.channel,
            Packet::Received(packet) => packet.get_channel(),
        }
    }

    pub fn get_index(&self) -> u32{
        match self{
            Packet::Constructed(packet) => packet.header.index,
            Packet::Received(packet) => packet.get_index(),
        }
    }

    pub fn get_data(&self) -> &[i16]{
        match self{
            Packet::Constructed(packet) => &packet.data,
            Packet::Received(packet) => packet.get_data(),
        }
    }

}

impl ConstructedPacket {
    pub fn new(channel: u32, index: u32, data: Vec<i16>) -> ConstructedPacket {
        ConstructedPacket{
            header: Header{
                identifier: ['w','p','p','\0'],
                channel,
                index,
            },
            data,
        }
    }

    pub fn encode(&self) -> Result<AlignedVec,Box<dyn Error>>{
        let bytes=rkyv::to_bytes::<_,4096>(self)?;

        println!("Identifier {:?}",self.header.identifier);
        println!("Bytes {:?}",bytes);

        let archive=rkyv::check_archived_root::<ConstructedPacket>(&bytes);
        println!("Archive {:?}",archive);

        Ok(bytes)
    }
}

impl OwnedArchivedPacket{
    fn decode(buf: Vec<u8>) -> Result<OwnedArchivedPacket,Box<dyn Error>>{
        let archive_result=rkyv::check_archived_root::<ConstructedPacket>(&buf);

        match archive_result{
            Ok(archive) => {
                println!("Decoded Archive {:?}",archive);
                if archive.header.identifier != ['w','p','p','\0']{
                    Err(format!("Invalid identifier: {:?}",archive.header.identifier).into())
                }else{
                    Ok(OwnedArchivedPacket{
                        buffer: buf,
                    })
                }
            },
            Err(e) => {
                Err(e.into())
            },
        }
    }

    fn get_channel(&self) -> u32{
        let archive=unsafe{
            rkyv::archived_root::<ConstructedPacket>(&self.buffer)
        };

        archive.header.channel
    }

    fn get_index(&self) -> u32{
        let archive=unsafe{
            rkyv::archived_root::<ConstructedPacket>(&self.buffer)
        };

        archive.header.index
    }

    fn get_data(&self) -> &ArchivedVec<i16>{
        let archive=unsafe{
            rkyv::archived_root::<ConstructedPacket>(&self.buffer)
        };

        &archive.data
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_packet(){
        let packet=Packet::new(0,0,vec![0,1,2,3,4,5,6,7,8,9]);
        let encoded=packet.encode().unwrap();


        let a=Packet::decode(encoded.into_vec()).unwrap();

        assert_eq!(a.get_data(), &vec![0,1,2,3,4,5,6,7,8,9]);

        assert_eq!(a.get_channel(), 0);
        assert_eq!(a.get_index(), 0);
    }

}

