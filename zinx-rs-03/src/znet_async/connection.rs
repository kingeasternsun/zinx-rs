#![allow(non_snake_case, dead_code)]
use crossbeam::channel;
use crossbeam_channel::select;
use tokio::io::AsyncReadExt;

// use std::net::Shutdown;
use bytes::Buf;
use bytes::BufMut;
use bytes::BytesMut;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

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
    is_closed: Arc<Mutex<bool>>,

    // 用来通知当前连接已经退出或停止
    exit_buff_chan: channel::Sender<bool>,
    receiver: channel::Receiver<bool>,
    handler_api: HandlerFn,
}

type HandlerFn =
    Arc<dyn Fn(&mut TcpStream, &mut [u8], usize) -> std::io::Result<usize> + Send + Sync>;

impl Connection {
    pub fn new(
        stream: TcpStream,
        socket_addr: SocketAddr,
        conn_id: u32,
        // sender: &channel::Sender<bool>,
        f: HandlerFn,
        //,
    ) -> Self {
        let (s, r) = channel::unbounded();
        Connection {
            conn: stream,
            socket_addr: socket_addr,
            conn_id: ConnID(conn_id),
            is_closed: Arc::new(Mutex::new(false)),
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
                    println!(
                        "{:?} recv {}",
                        self.conn_id,
                        String::from_utf8_lossy(&buf[..])
                    );
                    let _ = (self.handler_api)(&mut self.conn, &mut buf[..], n);
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
        // need fix
        // let start_read = self.start_read();
        // let mut r1 = self.receiver.clone();
        // let wait_close = self.wait_close();
        self.start_read().await;
    }

    async fn wait_close(&self) {
        let r1 = self.receiver.clone();
        select! {
            recv(r1)->msg => {
                println!("close {}",msg.unwrap());
                return
            } ,
        };
    }

    pub async fn stop(&mut self) {
        let mut close = self.is_closed.lock().await;
        if close.eq(&true) {
            return;
        }

        //TODO 如果用户注册了改链接的关闭回调业务，那么在此刻应该显示调用
        *close = true;

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

pub struct ConnectionSync {
    // 当前连接的tcpstream
    // read_half:tokio::io::ReadHalf<TcpStream>,
    // write_half:Arc<Mutex<tokio::io::WriteHalf<TcpStream>>>,
    conn: Arc<Mutex<TcpStream>>,
    // 对端地址
    socket_addr: SocketAddr,
    // 当前连接的ID 唯一
    conn_id: ConnID,
    // 当前连接的关闭状态
    is_closed: Arc<Mutex<bool>>,

    // 用来通知当前连接已经退出或停止
    exit_buff_chan: channel::Sender<bool>,
    receiver: channel::Receiver<bool>,
    handler_api: HandlerFnSync,
}

type HandlerFnSync =
    Arc<dyn Fn(Arc<Mutex<TcpStream>>, &mut [u8], usize) -> std::io::Result<usize> + Send + Sync>;

impl ConnectionSync {
    pub fn new(
        stream: TcpStream,
        socket_addr: SocketAddr,
        conn_id: u32,
        // sender: &channel::Sender<bool>,
        f: HandlerFnSync,
        //,
    ) -> Self {
        let (s, r) = channel::unbounded();
        // let (rt,wt) = tokio::io::split(stream);
        ConnectionSync {
            // read_half:rt,
            // write_half:Arc::new(Mutex::new(wt)),
            conn: Arc::new(Mutex::new(stream)),
            socket_addr: socket_addr,
            conn_id: ConnID(conn_id),
            is_closed: Arc::new(Mutex::new(false)),
            exit_buff_chan: s,
            receiver: r,
            handler_api: f,
        }
    }

    // 用于辅助 start_read
    async fn read_data<B: BufMut>(&self, buf: &mut B) -> std::io::Result<usize> {
        let mut s = self.conn.lock().await;
        s.read_buf(buf).await
    }

    // 用于辅助 start_read
    async fn write_data<B: Buf>(&self, buf: &mut B) -> std::io::Result<usize> {
        let mut s = self.conn.lock().await;
        s.write_buf(buf).await
    }

    async fn start_read(&self) {
        loop {
            let mut buf = BytesMut::with_capacity(256);
            match self.read_data(&mut buf).await {
                // !!! note if rev Ok(0) ,should break
                Ok(n) if n == 0 => break,
                Ok(n) => {
                    println!(
                        "{:?} recv {}",
                        self.conn_id,
                        String::from_utf8_lossy(&buf[..])
                    );

                    let _ = (self.handler_api)(self.conn.clone(), &mut buf[..], n);

                    match self.write_data(&mut buf).await {
                        Ok(n) => println!(
                            "{:?} write back {} to {}",
                            self.conn_id, n, self.socket_addr
                        ),
                        Err(_) => break,
                    };
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
    pub async fn start(self: &Arc<Self>) {
        println!("{:?} start", self.conn_id);

        let conn = Arc::clone(&self);

        // need fix 下面的代码只会接受一次就卡住不接受了 需要定位
        // let r = self.start_read();
        // let t = self.wait_close();
        // join!(r,t);

        tokio::spawn(async move { conn.wait_close().await });
        self.start_read().await;
    }

    async fn wait_close(&self) {
        let r1 = self.receiver.clone();
        select! {
            recv(r1)->msg => {
                println!("close {}",msg.unwrap());
                return
            } ,
        };
    }

    async fn stop_conn(&self) {
        let mut s = self.conn.lock().await;
        s.shutdown().await.unwrap();
    }

    pub async fn stop(&self) {
        let mut close = self.is_closed.lock().await;
        if close.eq(&true) {
            return;
        }

        //TODO 如果用户注册了改链接的关闭回调业务，那么在此刻应该显示调用
        *close = true;

        self.stop_conn().await;

        // 通知从 tcp stream读数据的业务关闭
        self.exit_buff_chan.send(true).unwrap();
        println!("[STOP] {:?} {}", self.get_conn_id(), self.remote_addr());
    }

    //从当前连接获取原始的tcp stream
    pub fn get_tcp_stream(&self) -> Arc<Mutex<TcpStream>> {
        self.conn.clone()
    }

    // 获取当前连接ID
    pub fn get_conn_id(&self) -> ConnID {
        self.conn_id
    }

    // 获取远程客户端地址信息
    pub fn remote_addr(&self) -> SocketAddr {
        self.socket_addr
    }
}
