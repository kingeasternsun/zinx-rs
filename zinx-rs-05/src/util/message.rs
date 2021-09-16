#![allow(non_snake_case, dead_code)]
#[derive(Clone, Debug)]
pub struct Message {
    Id: u32,        //消息的ID
    DataLen: usize, //消息的长度
    Data: Vec<u8>,  //消息的内容
}

impl Message {
    //创建一个Message消息包
    pub fn new(id: u32, data: Vec<u8>) -> Self {
        Message {
            Id: id,
            DataLen: 0,
            Data: data,
        }
    }

    //获取消息数据段长度
    pub fn GetDataLen(&self) -> usize {
        self.DataLen
    }

    //获取消息ID
    pub fn GetMsgId(&self) -> u32 {
        self.Id
    }

    //获取消息内容
    pub fn GetData(&self) -> Vec<u8> {
        self.Data.clone()
    }

    //设置消息数据段长度
    pub fn SetDataLen(&mut self, len: usize) {
        self.DataLen = len
    }

    //设计消息ID
    pub fn SetMsgId(&mut self, msgId: u32) {
        self.Id = msgId
    }

    //设计消息内容
    pub fn SetData(&mut self, data: Vec<u8>) {
        self.Data = data
    }

    pub fn reverse(&mut self) {
        self.Data.reverse()
    }
}
