参考golang版本 [五、Zinx的消息封装](https://www.kancloud.cn/aceld/zinx/1960224)

首先要包含如下依赖

```tom
[dependencies]
bytes = "1"
```

message 和 datapack 对于znet和znet_async 可以复用，所以把这两个部分的代码放入到util中，然后znet和zent_async里面引用即可。

# datapack

首先引入 use bytes::BufMut;  这样才能够在Pack中使用更加丰富的写入函数如put_u32,put_slice 等操作Vec；

再次引入 use bytes::Buf; 这样才能在UnPack中使用更加丰富的读取函数 get_u32 等操作。


在实现message的编码Pack和解码Unpack方法中，通过给 buffer 包装一层Cursor 来进行读取，这样Cursor的移动不会影响BytesMut内部的游标，所以执行
完 Datapack::check后，
1. 如果当前底层buffer里面的数据不足以构建一个完整的message，因为底层buffer的游标没有变，所以可以直接放弃这次解码，继续往buffer里面读入数据
2. 如果当前底层buffer里面的数据可以构建一个message，基于Cursor提取出message后，我们手动再将底层buffer的游标向后移动即可。


# 同步模型

对于同步模型,会尽可能的跟golang的设计保持一致
在本版本中，router 和  handler_api 只负责数据的处理，并不负责数据的发送工作
所以对于request的参数，都可以使用不可变引用进行优化。


# 异步模型

参考 https://tokio.rs/tokio/tutorial/framing 中的解码方式





