参考golang版本 [七、Zinx的读写分离模型](https://www.kancloud.cn/aceld/zinx/1960233)

当建立与客户端的套接字后，那么就会开启两个Thread  或 tokio::Spawn 两个Future 分别处理读数据业务和写数据业务，读写数据之间的消息通过一个Channel传递。




相比之前 
1. start_reader  start_writer  的入参改为了 & mut self， 使用
2. request IRequest 移除了Connction的引用
3. 同步模型 利用 Tcpstream::try_clone 得到新的Tcpstream，构建用于读和写的Connection，然后 同时启动 start_reader 和 start_writer 





另外之前Arc<Mutext<Connection>> 有潜在问题，例如reader 读取阻塞后，会一直lock住，导致出现死锁问题


async实现


tokio可以使用io::split 来把tcpStream 分为readerHalf和writerHalf两部分，这两个handler可以独立使用，也就是说可以调度到不同的task里面使用。

https://tokio.rs/tokio/tutorial/io#splitting-a-reader--writer

> Because io::split supports any value that implements AsyncRead + AsyncWrite and returns independent handles, internally io::split uses an Arc and a Mutex. This overhead can be avoided with TcpStream. TcpStream offers two specialized split functions.

> TcpStream::split takes a reference to the stream and returns a reader and writer handle. Because a reference is used, both handles must stay on the same task that split() was called from. This specialized split is zero-cost. There is no Arc or Mutex needed. TcpStream also provides into_split which supports handles that can move across tasks at the cost of only an Arc.

TcpStream::split 把 stream的引用返回为一个reader和writer handler。由于使用的是reference，所以这两个handler只能待在调用spit的同一个task中。这个spit函数是0开销的，没有使用Arc或Mutex。TcpStream同时还提供了一个 into_split 可以返回在task中move的handles，开销Arc。


```rust
    /// Splits a `TcpStream` into a read half and a write half, which can be used
    /// to read and write the stream concurrently.
    ///
    /// Unlike [`split`], the owned halves can be moved to separate tasks, however
    /// this comes at the cost of a heap allocation.
    ///
    /// **Note:** Dropping the write half will shut down the write half of the TCP
    /// stream. This is equivalent to calling [`shutdown()`] on the `TcpStream`.
    ///
    /// [`split`]: TcpStream::split()
    /// [`shutdown()`]: fn@crate::io::AsyncWriteExt::shutdown
    pub fn into_split(self) -> (OwnedReadHalf, OwnedWriteHalf) {
        split_owned(self)
    }
```

另外将Request中对Connection的引用去掉，仅保留Message类型的data