#![allow(non_snake_case)]
use std::io::Read;

use crate::util::Error;
use crate::util::Message;
use crate::ziface::IDataPack;
use bytes::Buf;
use bytes::BufMut;

pub struct DataPack;
impl DataPack {
    pub fn check(src: &mut std::io::Cursor<&[u8]>) -> Result<(), Error> {
        // println!("remains {}",src.remaining());

        if src.remaining() < 8 {
            return Err(Error::Incomplete);
        }

        src.advance(4);

        let dataLen = src.get_u32() as usize;
        if src.remaining() < dataLen {
            return Err(Error::Incomplete);
        }

        Ok(())
    }
    pub(crate) fn Unpack(src: &mut std::io::Cursor<&[u8]>) -> Result<Message, Error> {
        if src.remaining() < 4 {
            return Err(Error::Incomplete);
        }

        let msgID = src.get_u32();
        if src.remaining() < 4 {
            return Err(Error::Incomplete);
        }
        let dataLen = src.get_u32() as usize;
        if src.remaining() < dataLen {
            return Err(Error::Incomplete);
        }

        let mut data = vec![0; dataLen];
        let _ = src.read_exact(&mut data[..]);
        Ok(Message::new(msgID, data))
    }

    // 限制只能在当前create中调用
    pub(crate) fn Pack(msg: &Message) -> Result<Vec<u8>, std::io::Error> {
        let mut buf = vec![];
        buf.put_u32(msg.GetMsgId());
        buf.put_u32(msg.GetDataLen() as u32);
        buf.put_slice(&(msg.GetData())[..]);

        Ok(buf)
    }
}
impl IDataPack for DataPack {
    type Msg = Message;

    fn GetHeadLen() -> u32 {
        8
    }

    fn Pack(msg: &Self::Msg) -> Result<Vec<u8>, std::io::Error> {
        DataPack::Pack(msg)
    }

    fn Unpack(src: &mut std::io::Cursor<&[u8]>) -> Result<Self::Msg, Error> {
        DataPack::Unpack(src)
    }
}
