use crate::ziface::IRquest;
use crate::znet_async::ConnectionSync;
use std::sync::Arc;
#[derive(Clone)]
pub struct Request {
    conn: Arc<ConnectionSync>, // 已经和客户端建立好的 连接
    data: Vec<u8>,              //客户端请求的数据
}

impl Request {
    pub fn new(con: Arc<ConnectionSync>, data: &[u8]) -> Self {
        Request {
            conn: Arc::clone(&con),
            data: data.to_vec(),
        }
    }
}

impl IRquest for Request {
    type Conn = ConnectionSync; // 对于某个 Connection 的 Request，对应的 IRquest 只有一种比较合理
                                // 获取请求连接信息
    fn get_connection(&self) -> Arc<Self::Conn> {
        Arc::clone(&self.conn)
    }
  
    // 获取请求消息的数据
    fn get_data(&self) ->&[u8] {
        &self.data[..]
    }
}
