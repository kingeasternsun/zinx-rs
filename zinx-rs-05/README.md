参考golang版本 [五、Zinx的消息封装](https://www.kancloud.cn/aceld/zinx/1960224)

首先要包含如下依赖

```tom
[dependencies]
serde = { version = "1.0.104", features = ["derive"] }
structopt      = "0.3.11"
structopt-toml = "0.5.0"
toml           = "0.5.6"
```

message 和 datapack 对于znet和znet_async 可以复用，所以把这两个部分的代码放入到util中，然后znet和zent_async里面引用即可。

# datapack

首先引入 use bytes::BufMut;  这样才能够在Pack中使用更加丰富的写入函数如put_u32,put_slice 等操作Vec；

再次引入  use bytes::Buf; 这样才能在UnPack中使用更加丰富的读取函数 get_u32 等操作。

给DataPack 实现 Unpack时候，声明为 pub(crate), 这样就可以直接 DataPack::unpack进行调用了。
```rust
impl DataPack {

    pub(crate) fn Unpack(src: &mut std::io::Cursor<&[u8]>) -> Result<Message, Error> {
        ...
    }
}
```

# 同步模型

对于同步模型,会尽可能的跟golang的设计保持一致


# 异步模型

参考 https://tokio.rs/tokio/tutorial/framing 中的解码方式





