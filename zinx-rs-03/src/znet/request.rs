
use std::sync::Arc;
use crate::znet::connection::ConnectionSync;
use crate::ziface::IRquest;
pub struct Request<'a> {
    conn: Arc<ConnectionSync>, // 已经和客户端建立好的 连接
    data: &'a [u8],            //客户端请求的数据
}

impl <'a> IRquest<'a> for Request<'a> {
    type Conn = ConnectionSync;// 对于某个 Connection 的 Request，对应的 IRquest 只有一种比较合理
    // 获取请求连接信息
    fn get_connection(&self) -> Arc<ConnectionSync> {
        Arc::clone(&self.conn)
    }
    // 获取请求消息的数据
    fn get_data(&'a self) -> &'a [u8] {
        self.data
    }
}
