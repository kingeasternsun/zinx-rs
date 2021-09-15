use crate::ziface::IRquest;
use crate::znet::connection::ConnectionSync;
use std::sync::Arc;
///  等对生命周期掌握的更加熟悉后再考虑这个
// pub struct Request<'a> {
//     conn: Arc<ConnectionSync>, // 已经和客户端建立好的 连接
//     data: &'a [u8],            //客户端请求的数据
// }

// impl<'a> IRquest<'a> for Request<'a> {
//     type Conn = ConnectionSync; // 对于某个 Connection 的 Request，对应的 IRquest 只有一种比较合理
//                                 // 获取请求连接信息
//     fn get_connection(&self) -> Arc<ConnectionSync> {
//         Arc::clone(&self.conn)
//     }
//     // 获取请求消息的数据
//     fn get_data(&'a self) -> &'a [u8] {
//         self.data
//     }
// }
#[derive(Clone)]
pub struct Request {
    conn: Arc<ConnectionSync>, // 已经和客户端建立好的 连接
    data: String,              //客户端请求的数据
}

impl Request {
    pub fn new(con: Arc<ConnectionSync>, data: String) -> Self {
        Request {
            conn: Arc::clone(&con),
            data: data,
        }
    }
}

impl IRquest for Request {
    type Conn = ConnectionSync; // 对于某个 Connection 的 Request，对应的 IRquest 只有一种比较合理
                                // 获取请求连接信息
    fn get_connection(&self) -> Arc<ConnectionSync> {
        Arc::clone(&self.conn)
    }
    // 获取请求消息的数据
    fn get_data(&self) -> String {
        self.data.clone()
    }
}
