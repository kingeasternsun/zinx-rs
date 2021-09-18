use crate::util::ConnID;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
pub trait Iconnection {
    // 启动连接，让当前连接开始工作
    fn start(self: &Arc<Self>);
    fn stop(&self);
    //从当前连接获取原始的tcp stream
    fn get_tcp_stream(&self) -> Arc<Mutex<TcpStream>>;
    // 获取当前连接ID
    fn get_conn_id(&self) -> ConnID;
    // 获取远程客户端地址信息
    fn remote_addr(&self) -> SocketAddr;
}
