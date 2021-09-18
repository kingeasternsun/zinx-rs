#![allow(non_snake_case, dead_code)]
use crossbeam::channel;
use crossbeam::scope;
use crossbeam_channel::select;
use std::io::{Cursor, Read, Write};

use bytes::{Buf, BufMut, BytesMut};
use std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::util::*;
use crate::ziface::Iconnection;
use crate::znet::Request;
#[derive(Clone)]
pub struct ConnectionSync {
    // 当前连接的tcpstream
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

    msg_sx:channel::Sender<Message>,
    msg_rx:channel::Receiver<Message>,
    // handler_api: HandlerFnSync,
    Router: Arc<MsgHandle<Request>>,
}

type HandlerFnSync =
    Arc<dyn Fn(Arc<Mutex<TcpStream>>, &Message) -> std::io::Result<Message> + Send + Sync>;

impl ConnectionSync {
    pub fn new(
        stream: TcpStream,
        socket_addr: SocketAddr,
        conn_id: u32,
        // sender: &channel::Sender<bool>,
        // f: HandlerFnSync,
        router: Arc<MsgHandle<Request>>,
        //,
    ) -> Self {
        let (s, r) = channel::unbounded();
        let (sx, rx) = channel::unbounded();
        ConnectionSync {
            conn: Arc::new(Mutex::new(stream)),
            socket_addr,
            conn_id: ConnID::new(conn_id),
            is_closed: Arc::new(Mutex::new(false)),
            exit_buff_chan: s,
            receiver: r,
            msg_rx:rx,
            msg_sx:sx,
            // handler_api: f,
            Router: router,
        }
    }

    ///  从buffer中解析 message
    /// 返回 Ok(None) 表示 buffer中数据不足
    fn parse_message(
        &self,
        buffer: &mut BytesMut,
    ) -> std::result::Result<Option<Message>, Error> {
        use crate::util::Error::Incomplete;
        let mut buf = Cursor::new(&buffer[..]);

        // The first step is to check if enough data has been buffered to parse
        // a single frame. This step is usually much faster than doing a full
        // parse of the frame, and allows us to skip allocating data structures
        // to hold the frame data unless we know the full frame has been
        // received.
        // 先判断buffer中是否有足够的数据解码message，如果不够就往buffer里面读入新数据
        // check步骤通常比parse要快，因为check不需要完全的进行解码，对于message包含非常多的复杂字段的情况，是非常快的
        // 如果不先进行check，那么对于数据不完整的情况，就会出现parse了一部分然后放弃，比较浪费
        match DataPack::check(&mut buf) {
            Ok(_) => {
                // Reset the position to zero before passing the cursor to
                // `Frame::parse`.
                // 重置position归零，用于下面的Unpack操作，因为 Unpack也是基于Cursor的
                buf.set_position(0);

                // Parse the frame from the buffer. This allocates the necessary
                // structures to represent the frame and returns the frame
                // value.
                //
                // If the encoded frame representation is invalid, an error is
                // returned. This should terminate the **current** connection
                // but should not impact any other connected client.
                let frame = DataPack::Unpack(&mut buf)?;

                // The `check` function will have advanced the cursor until the
                // end of the frame. Since the cursor had position set to zero
                // before `Frame::check` was called, we obtain the length of the
                // frame by checking the cursor position.
                // 获取到了当前message的长度
                // 跟 minitokio的实现不太一样，放在这里，获取的len才是message的字节流的真正长度
                let len = buf.position() as usize;

                // Discard the parsed data from the read buffer.
                //
                // When `advance` is called on the read buffer, all of the data
                // up to `len` is discarded. The details of how this works is
                // left to `BytesMut`. This is often done by moving an internal
                // cursor, but it may be done by reallocating and copying data.
                // message已经读取出来了，所以需要把底层的buffer的游标也移动一下
                buffer.advance(len);

                // Return the parsed frame to the caller.
                Ok(Some(frame))
            }
            // There is not enough data present in the read buffer to parse a
            // single frame. We must wait for more data to be received from the
            // socket. Reading from the socket will be done in the statement
            // after this `match`.
            //
            // We do not want to return `Err` from here as this "error" is an
            // expected runtime condition.
            Err(Incomplete) => Ok(None),
            // An error was encountered while parsing the frame. The connection
            // is now in an invalid state. Returning `Err` from here will result
            // in the connection being closed.
            Err(e) => Err(e),
        }
    }

    // 用于辅助 read_message
    fn read_data<B: BufMut>(&self, buffer: &mut B) -> std::io::Result<usize> {
        let mut buf = vec![0; 256];

        let mut conn = self.conn.lock().unwrap();
        let n = conn.read(&mut buf)?;
        buffer.put_slice(&buf[..n]);
        Ok(n)
    }

    // 用于辅助 start_reader
    fn write_data(&self, buf: &[u8]) -> std::io::Result<()> {
        println!("to lock");
        let mut s = self.conn.try_lock().unwrap();
        println!("write finish");
        s.write_all(buf)
        
    }

    /// 从 buffer 中解析message，如果解析成功就返回,数据不充足就从tcp stream中读入新数据到buffer
    /// 返回Ok(None) 表示客户端已经完成发送，合法的关闭了
    fn read_message(
        &self,
        buffer: &mut BytesMut,
    ) -> std::result::Result<Option<Message>, Error> {
        loop {
            if let Some(msg) = self.parse_message(buffer)? {
                return Ok(Some(msg));
            }

            // 读取的数据不完全，不足以解析出来 ,就继续往buffer里面读入数据
            // There is not enough buffered data to read a frame. Attempt to
            // read more data from the socket.
            //
            // On success, the number of bytes is returned. `0` indicates "end
            // of stream".
            match self.read_data(buffer) {
                Err(err) => {
                    println!("{}", err);
                    return Err(Error::Other(Box::new(err)));
                }
                Ok(0) => {
                    // The remote closed the connection. For this to be a clean
                    // shutdown, there should be no data in the read buffer. If
                    // there is, this means that the peer closed the socket while
                    // sending a frame.
                    if buffer.is_empty() {
                        println!("READ 0");
                        return Ok(None);
                    } else {
                        return Err("connection reset by peer".into());
                    }
                }
                Ok(n) => {
                    println!("REVE {}", n);
                }
            }
        }
    }

    pub fn start_reader(&self ) {
        let mut buffer = BytesMut::with_capacity(1024);
        loop {
            match self.read_message(&mut buffer) {
                Err(err) => {
                    println!("{:?}{}", self.conn_id, err);
                    break;
                }

                Ok(None) => break,
                Ok(Some(msg)) => {
                    let request = Request::new(Arc::new(self.clone()), msg);

                    if let Some(res) = self.Router.DoMsgHandler(&request){
  
                        // let data = DataPack::Pack(&res.data).unwrap();
    
                        match self.msg_sx.send(res.data) {
                            Ok(_) => {
                                println!("write back  to chanel");
                                // self.Router.post_hander(&request);
                            }
                            Err(_) => break,
                        };
                    }

     
                }
            }
        }

        self.stop();
    }

    pub fn start_writer(&self){
        loop{

            //TODO select receiver
            if let Ok(msg) = self.msg_rx.recv(){
                println!("rev {}",msg);
                let data = DataPack::Pack(&msg).unwrap();
    
                match self.write_data(&data) {
                    Ok(_) => {
                        println!("WRITER {:?} write back  to {}", self.conn_id, self.socket_addr);
                    }
                    Err(err) => {
                        println!("{}",err);
                        break;
                    },
                };
            }else{
                break
            }
            // select!{
            //     recv(self.msg_rx)-> msg => {
            //         match msg{

            //         }
            //     },
            //     recv(self.exit_buff_chan)->msg =>{
            //         return 
            //     },
            // }
        }
    }

    
}

impl ConnectionSync {
    // 启动连接，让当前连接开始工作
    pub fn start(&self) {
        println!("{:?} start", self.conn_id);

        scope(|scope| {
            // Spawn a thread that receives a message and then sends one.
            scope.spawn(|_| {
                self.start_reader();
            });

            scope.spawn(|_| {
                self.start_writer();
            });
        
        }).unwrap();

        let r1 = self.receiver.clone();

        select! {
            recv(r1)->msg => {
                println!("close {}",msg.unwrap());

            } ,
        }
    }

    fn stop(&self) {
        let mut close = self.is_closed.lock().unwrap();
        if close.eq(&true) {
            return;
        }

        //TODO 如果用户注册了改链接的关闭回调业务，那么在此刻应该显示调用
        *close = true;
        match self.conn.lock().unwrap().shutdown(Shutdown::Both) {
            Ok(_) => {}
            Err(err) => println!("{}", err),
        }

        // 通知从 tcp stream读数据的业务关闭
        self.exit_buff_chan.send(true).unwrap();
        println!("[STOP] {:?} {}", self.get_conn_id(), self.remote_addr());
    }

    //从当前连接获取原始的tcp stream
    fn get_tcp_stream(&self) -> Arc<Mutex<TcpStream>> {
        self.conn.clone()
    }

    // 获取当前连接ID
    fn get_conn_id(&self) -> ConnID {
        self.conn_id
    }

    // 获取远程客户端地址信息
    fn remote_addr(&self) -> SocketAddr {
        self.socket_addr
    }
}
