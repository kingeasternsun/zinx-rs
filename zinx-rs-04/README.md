参考golang版本 [四、Zinx的全局配置](https://www.kancloud.cn/aceld/zinx/1960221)
在当前项目rust实现中，使用crate [structopt-toml](https://crates.io/crates/structopt-toml)读取toml格式的配置文件


首先要包含如下依赖

```tom
[dependencies]
serde = { version = "1.0.104", features = ["derive"] }
structopt      = "0.3.11"
structopt-toml = "0.5.0"
toml           = "0.5.6"
```

# 同步模型

对于同步模型，我们使用标准库的 

```rust
use std::fs::File;
use std::io::Read;
```
来读取配置文件的内容到buf中。

# 异步模型

参考 https://tokio.rs/tokio/tutorial/io

使用tokio库中的

```rust
use tokio::io::AsyncReadExt;
use tokio::fs::File;
```

来读取配置文件的内容到buf中。




