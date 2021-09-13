use std::sync::Arc;
use crate::ziface::Iconnection;

pub trait IRquest<'a,T> {
    // 获取请求连接信息
    fn get_connection(&self) -> Arc<T> where T:Iconnection;
    // 获取请求消息的数据
    fn get_data(&'a self) -> &'a [u8];
}
