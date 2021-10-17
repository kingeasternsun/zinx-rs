参考 golang 版本 [Zinx-V0.1-基础Server](https://www.kancloud.cn/aceld/zinx/1960213)，对于server部分，分别使用标准库和Tokio库各自进行了开发，分别对应[znet](https://github.com/kingeasternsun/zinx-rs/tree/main/zinx-rs-01/src/znet) 和 [znet_async](https://github.com/kingeasternsun/zinx-rs/tree/main/zinx-rs-01/src/znet_async)。

# znet
标准库实现时，对stream的读写需要引入下面两个trait
```rust
use std::io::{Read, Write};
```

tokio 实现时，对stream的读写需要引入下面两个trait
```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
```


# 运行方式

对于同步实现的server
```shell
cargo run --bin=server
```

对于异步实现的server
```shell
cargo run --bin=server-async
```

启动 tcp 请求客户端
```shell
cargo run --bin=cli
```