use std::error::Error;

use rkyv::{
    Archive,
    Serialize,
    Deserialize,
    ser::{
        serializers::{BufferSerializer, AllocSerializer},
        Serializer,
    },
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
struct ConstructedPacket{
    pub header: Header,
    pub data: Vec<i16>,
}

struct OwnedArchivedPacket<'a>{
    archive: Option<&'a ArchivedConstructedPacket>,
    buffer: Vec<u8>,
}

pub enum Packet<'a>{
    Constructed(ConstructedPacket),
    Received(OwnedArchivedPacket<'a>),
}

impl<'a> Packet<'a>{
    pub fn new(channel: u32, index: u32, data: Vec<i16>) -> Packet<'a>{
        Packet::Constructed(ConstructedPacket::new(channel,index,data))
    }

    pub fn encode(&self, buf: &mut Vec<u8>) -> Result<(),Box<dyn Error>>{
        match self{
            Packet::Constructed(packet) => packet.encode(buf),
            Packet::Received(packet) => {
                Err("Sending a received packet".into())
            },
        }
    }

    pub fn decode(buf: Vec<u8>) -> Result<Packet<'a>,Box<dyn Error>>{
        let p=OwnedArchivedPacket::decode(buf)?;

        Ok(Packet::Received(p))
    }

    pub fn get_channel(&self) -> u32{
        match self{
            Packet::Constructed(packet) => packet.header.channel,
            Packet::Received(packet) => packet.archive.unwrap().header.channel,
        }
    }

    pub fn get_index(&self) -> u32{
        match self{
            Packet::Constructed(packet) => packet.header.index,
            Packet::Received(packet) => packet.archive.unwrap().header.index,
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

    pub fn encode(&self, buf: &mut Vec<u8>) -> Result<(),Box<dyn Error>>{
        let bytes=rkyv::to_bytes::<_,4096>(self)?;
        buf.copy_from_slice(&bytes);

        Ok(())
    }
}

impl<'a> OwnedArchivedPacket<'a>{
    fn decode(buf: Vec<u8>) -> Result<OwnedArchivedPacket<'a>,Box<dyn Error>>{
        // let archive=rkyv::check_archived_root::<ConstructedPacket>(&buf)?;

        // Ok(OwnedArchivedPacket{
        //     archive: archive,
        //     buffer: buf,
        // })
        let mut ar=OwnedArchivedPacket{
            archive: None,
            buffer: buf,
        };

        let archive=rkyv::check_archived_root::<ConstructedPacket>(&ar.buffer)?;
        ar.archive=Some(archive);

        Ok(ar)
    }
}

#[cfg(test)]
mod test{
    use super::*;


    #[test]
    fn test_packet(){
        let packet=ConstructedPacket::new(0,0,vec![0,1,2,3,4,5,6,7,8,9]);
        let serialized=rkyv::to_bytes::<_,4096>(&packet).unwrap();
        let archive=rkyv::check_archived_root::<ConstructedPacket>(&serialized).unwrap();

        let a=&archive.data;
        assert_eq!(a, &vec![0,1,2,3,4,5,6,7,8,9]);

        let b=&archive.header;
        assert_eq!(b.identifier, ['w','p','p','\0']);
        assert_eq!(b.channel, 0);
        assert_eq!(b.index, 0);

        assert_eq!(archive, &packet);
    }

}

