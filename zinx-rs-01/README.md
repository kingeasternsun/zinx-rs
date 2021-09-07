对应 https://www.kancloud.cn/aceld/zinx/1960213

同步实现时，对stream的读写需要引入下面两个trait
```rust
use std::io::{Read, Write};
```

aysnc 实现时，对stream的读写需要引入下面两个trait
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