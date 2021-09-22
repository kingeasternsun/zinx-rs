#![allow(dead_code, non_snake_case)]
use crate::util::Message;
use crate::ziface::IRquest;
use crate::znet_async::ConnectionSync;
use std::sync::Arc;
#[derive(Clone)]
pub struct Request {
    // conn: Arc<ConnectionSync>, // 已经和客户端建立好的 连接
    pub data: Message, //客户端请求的数据
}

impl Request {
    pub fn new(data: Message) -> Self {
        Request {
            // conn: Arc::clone(&con),
            data,
        }
    }
}

// impl IRquest for Request {
//     type Conn = ConnectionSync; // 对于某个 Connection 的 Request，对应的 IRquest 只有一种比较合理
//                                 // 获取请求连接信息
//     fn get_connection(&self) -> Arc<Self::Conn> {
//         Arc::clone(&self.conn)
//     }
//     // 获取请求消息的数据
//     fn get_data(&self) -> String {
//         unsafe { String::from_utf8_unchecked(self.data.GetData()) }
//     }
// }

impl IRquest for Request {
    // type Conn = ConnectionSync; // 对于某个 Connection 的 Request，对应的 IRquest 只有一种比较合理
    //                             // 获取请求连接信息
    // fn get_connection(&self) -> Arc<Self::Conn> {
    //     Arc::clone(&self.conn)
    // }
    // 获取请求消息的数据
    fn get_data(&self) -> Message {
        self.data.clone()
    }

    // 获取消息的id
    fn get_msgID(&self) -> u32 {
        self.data.GetMsgId()
    }
}
