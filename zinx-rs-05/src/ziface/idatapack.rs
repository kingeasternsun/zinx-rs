#![allow(non_snake_case, dead_code)]
use crate::util::Error;
use std::io::Cursor;
pub trait IDataPack {
    type Msg;
    /// 获取包头长度方法
    fn GetHeadLen() -> u32;
    /// 封包方法
    fn Pack(msg: &Self::Msg) -> Result<Vec<u8>, std::io::Error>;
    /// 拆包方法
    fn Unpack(src: &mut Cursor<&[u8]>) -> Result<Self::Msg, Error>;
}
