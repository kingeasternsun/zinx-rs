// use crate::ziface::Iconnection;
use std::sync::Arc;

// TODO 等对生命周期掌握的更加熟悉后再考虑这种
// pub trait IRquest<'a> {
//     // 关联 Connection
//     type Conn;
//     // 获取请求连接信息
//     fn get_connection(&self) -> Arc<Self::Conn>;
//     // 获取请求消息的数据
//     fn get_data(&'a self) -> &'a [u8];
// }

pub trait IRquest {
    // 关联 Connection
    type Conn;
    // 获取请求连接信息
    fn get_connection(&self) -> Arc<Self::Conn>;
    // 获取请求消息的数据
    fn get_data(&self) -> &[u8];
}
