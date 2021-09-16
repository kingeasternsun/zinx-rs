#![allow(non_snake_case, dead_code)]
use crossbeam::channel;
use crossbeam_channel::select;
use tokio::io::AsyncReadExt;

// use std::net::Shutdown;
use bytes::Buf;
use bytes::BufMut;
use bytes::BytesMut;
use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::util::ConnID;
use crate::util::DataPack;
use crate::util::Error;
use crate::util::Message;
use crate::znet_async::Request;
use crate::znet_async::RouterSync;

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
    Router: RouterSync,
    // buffer: BytesMut,
}

type HandlerFnSync =
    Arc<dyn Fn(Arc<Mutex<TcpStream>>, &Message) -> std::io::Result<Message> + Send + Sync>;

impl ConnectionSync {
    pub fn new(
        stream: TcpStream,
        socket_addr: SocketAddr,
        conn_id: u32,
        // sender: &channel::Sender<bool>,
        f: HandlerFnSync,
        router: RouterSync,
        //,
    ) -> Self {
        let (s, r) = channel::unbounded();
        // let (rt,wt) = tokio::io::split(stream);
        ConnectionSync {
            // read_half:rt,
            // write_half:Arc::new(Mutex::new(wt)),
            conn: Arc::new(Mutex::new(stream)),
            socket_addr: socket_addr,
            conn_id: ConnID::new(conn_id),
            is_closed: Arc::new(Mutex::new(false)),
            exit_buff_chan: s,
            receiver: r,
            handler_api: f,
            Router: router,
            //
        }
    }

    // 用于辅助 start_read
    async fn read_data<B: BufMut>(&self, buf: &mut B) -> std::io::Result<usize> {
        let mut s = self.conn.lock().await;
        s.read_buf(buf).await
    }

    // 用于辅助 start_read
    async fn write_data(&self, buf: & [u8]) -> std::io::Result<()> {
        let mut s = self.conn.lock().await;
        s.write_all(buf).await
    }

    // 从buffer中解析 message
    async fn parse_message(
        self: &Arc<Self>,
        buf: &mut BytesMut,
    ) -> std::result::Result<Option<Message>, Error> {
        use crate::util::Error::Incomplete;
        let mut buf = Cursor::new(&buf[..]);
        match DataPack::Unpack(&mut buf) {
            Ok(msg) => Ok(Some(msg)),
            Err(Incomplete) => Ok(None),
            Err(err) => Err(err),
        }
    }

    // 从 tcp stream 中解析message，如果解析成功就返回
    async fn read_message(
        self: &Arc<Self>,
        buffer: &mut BytesMut,
    ) -> std::result::Result<Option<Message>, Error> {
        loop {
            if let Some(msg) = self.parse_message(buffer).await? {
                return Ok(Some(msg));
            }

            // 读取的数据不完全，不足以解析出来 ,就继续往buffer里面读入数据
            // There is not enough buffered data to read a frame. Attempt to
            // read more data from the socket.
            //
            // On success, the number of bytes is returned. `0` indicates "end
            // of stream".
            match self.read_data(buffer).await {
                Err(err) => return Err(Error::Other(Box::new(err))),
                Ok(0) => {
                    // The remote closed the connection. For this to be a clean
                    // shutdown, there should be no data in the read buffer. If
                    // there is, this means that the peer closed the socket while
                    // sending a frame.
                    if buffer.is_empty() {
                        return Ok(None);
                    } else {
                        return Err("connection reset by peer".into());
                    }
                }
                Ok(_) => {}
            }
        }
    }

    async fn start_read(self: &Arc<Self>) {
        let mut buffer = BytesMut::with_capacity(1024);
        loop {
            match self.read_message(&mut buffer).await {
                Err(err) => {
                    println!("{:?}{}", self.conn_id, err);
                    break;
                }

                Ok(None) => break,
                Ok(Some(msg)) => {
                    let request = Request::new(Arc::clone(self), msg);

                    self.Router.pre_handler(request.clone());

                    let res = (self.handler_api)(self.conn.clone(), &request.data).unwrap();

                    println!("{:?}", res);

                    let data = DataPack::Pack(&res).unwrap();

                    match self.write_data(& data).await {
                        Ok(n) => println!(
                            "{:?} write back  to {}",
                            self.conn_id, self.socket_addr
                        ),
                        Err(_) => break,
                    };
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
