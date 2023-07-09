use std::{str::Bytes, vec};

use zerocopy::{AsBytes, FromBytes,Unaligned,ByteSlice,LayoutVerified, ByteSliceMut,U32,
byteorder::NetworkEndian, I16, NativeEndian};

#[derive(FromBytes, AsBytes, Unaligned)]
#[repr(C)]
pub struct Header{
    pub identifier: [u8;4],
    pub channel: U32<NetworkEndian>,
    pub index: U32<NetworkEndian>,
}


struct SerdePacket<B: ByteSlice>{
    header: LayoutVerified<B,Header>,
    data: B,
}

impl<B: ByteSlice> SerdePacket<B> {
    pub fn decode(bytes: B) -> Option<SerdePacket<B>> {
        let (header,body)=LayoutVerified::new_from_prefix(bytes)?;

        Some(SerdePacket { header, data: body })
    }

    pub fn encode_to_buf(&self, buf: &mut impl ByteSliceMut) -> Result<(),String> {
        let hl=self.header.as_bytes().len();
        let dl=self.data.len();

        if buf.len()!=hl+dl {
            return Err(format!("Buffer length {} is not equal to header length {} + data length {}",buf.len(),hl,dl));
        }

        buf[..hl].copy_from_slice(self.header.as_bytes());
        buf[hl..].copy_from_slice(self.data.as_bytes());

        Ok(())
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf_vec=vec![0;self.len()];
        let mut buf=buf_vec.as_mut_slice();
        self.encode_to_buf(&mut buf).unwrap();
        buf_vec
    }

    pub fn len(&self) -> usize {
        self.header.as_bytes().len()+self.data.len()
    }

}

pub struct Packet{
    pub header: Header,
    pub data: Vec<i16>,
}

impl Packet{
    pub fn new(channel: u32, index: u32, data: Vec<i16>) -> Packet {
        Packet{
            header: Header{
                identifier: [b'w',b'p',b'p',0],
                channel: channel.into(),
                index: index.into(),
            },
            data,
        }
    }

    fn s(&self){
        let mut buf=self.data.as_bytes_mut();
        assert!(buf.len()%2==0);

        for i in 0..buf.len()/2{
            let mut num_buf: &[u8;2]=&buf[i*2..][..2].try_into().unwrap();
            let num=I16::<NativeEndian>::from_bytes(*num_buf);
            let num=I16::<NetworkEndian>::from(num.into());
            num_buf.copy_from_slice(num.as_bytes());
        }

        let serdep=SerdePacket::<&[u8]>{
            header: LayoutVerified::<&[u8],Header>::new(self.header.as_bytes()).unwrap(),
            data: self.data.into(),
        };

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_works() {
        let bytes:[u8; 15]  = [
            b'w',b'p',b'p',0,        // identifier
            0,0,0,1,                    // channel
            0,0,0,1,                    // index
            0x05,0x06,0x07,             // body
        ];

        let packet=SerdePacket::decode(&bytes[..]).unwrap();

        assert!(packet.header.identifier.get(..).unwrap()==b"wpp\0");
        assert!(packet.header.channel.get()==1);
        assert!(packet.header.index.get()==1);
        assert!(packet.data.get(..).unwrap()==&[0x05,0x06,0x07][..]);

        let encoded=packet.encode();
        assert!(encoded.as_slice()==bytes);
    }

    #[test]
    fn packet_works() {
        let packet=Packet::new(1,1,vec![0x05,0x06,0x07]);

        packet.s();
    }

}
