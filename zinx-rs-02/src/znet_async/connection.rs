#![allow(non_snake_case)]
use crossbeam::channel;
// use crossbeam_channel::select;
use tokio::io::AsyncReadExt;

// use std::net::Shutdown;
use tokio::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use bytes::BytesMut;

/// connection
///

#[derive(Debug, Clone, Copy)]
pub struct ConnID(u32);
pub struct Connection {
    // 当前连接的tcpstream
    conn: TcpStream,
    // 对端地址
    socket_addr: SocketAddr,
    // 当前连接的ID 唯一
    conn_id: ConnID,
    // 当前连接的关闭状态
    // is_closed: Arc<Mutex<bool>>,

    // 用来通知当前连接已经退出或停止
    exit_buff_chan: channel::Sender<bool>,
    receiver: channel::Receiver<bool>,
    handler_api: Arc<dyn Fn(& mut TcpStream, &mut[u8], usize) -> std::io::Result<usize> + Send + Sync>,
}

impl Connection {
    pub fn new(
        stream: TcpStream,
        socket_addr: SocketAddr,
        conn_id: u32,
        // sender: &channel::Sender<bool>,
        f: Arc<dyn Fn(& mut TcpStream, &mut[u8], usize) -> std::io::Result<usize> + Send + Sync>,
        //,
    ) -> Self {
        let (s, r) = channel::unbounded();
        Connection {
            conn: stream,
            socket_addr: socket_addr,
            conn_id: ConnID(conn_id),
            // is_closed: Arc::new(Mutex::new(false)),
            exit_buff_chan: s,
            receiver: r,
            handler_api: f,
        }
    }

    async fn start_read(&mut self) {
        loop {
            let mut buf = BytesMut::with_capacity(256);
            match self.conn.read_buf(&mut buf).await {
                // !!! note if rev Ok(0) ,should break
                Ok(n) if n == 0 => break,
                Ok(n) => {
                    println!("{:?} recv {}",self.conn_id, String::from_utf8_lossy(&buf[..]));
                    let _ = (self.handler_api)(& mut self.conn, & mut buf[..], n);
                    match self.conn.write(&buf[..n]).await {
                        Ok(n) => println!("{:?} write back{}", self.conn_id, n),
                        Err(_) => break,
                    }
                }
                Err(err) => {
                    println!("{:?}{}", self.conn_id, err);
                    break;
                }
            }
        }

        self.stop().await;
    }

    // 启动连接，让当前连接开始工作
    pub async fn start(&mut self) {
        println!("{:?} start", self.conn_id);
        self.start_read().await;

        // select! {
        //     recv(r1)->msg => {
        //         println!("close {}",msg.unwrap());
        //         return
        //     } ,
        // }
    }

    pub async fn stop(&mut self) {
        // let mut close = self.is_closed.lock().unwrap();
        // if close.eq(&true) {
        //     return;
        // }

        // //TODO 如果用户注册了改链接的关闭回调业务，那么在此刻应该显示调用
        // *close = true;
      
        self.conn.shutdown().await.unwrap();

        // 通知从 tcp stream读数据的业务关闭
        self.exit_buff_chan.send(true).unwrap();
        println!("{}", "[STOP]");
    }

    //从当前连接获取原始的tcp stream
    // pub fn get_tcp_stream(&self) -> TcpStream {
    //     self.conn.unwrap()
    // }

    // 获取当前连接ID
    pub fn get_conn_id(&self) -> ConnID {
        self.conn_id
    }

    // 获取远程客户端地址信息
    pub fn remote_addr(&self) -> SocketAddr {
        self.socket_addr
    }
}
