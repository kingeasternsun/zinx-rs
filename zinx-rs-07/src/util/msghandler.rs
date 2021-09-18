#![allow(non_snake_case, dead_code)]
use crate::IRouter;
use crate::IRquest;
use std::collections::HashMap;

pub struct MsgHandle<R> {
    //存放每个MsgId 所对应的处理方法的map属性
    Apis: HashMap<u32, Box<dyn IRouter<R = R> + Send + Sync>>,
}

impl<R> MsgHandle<R> {
    pub fn new() -> Self {
        MsgHandle {
            Apis: HashMap::new(),
        }
    }

    /// 马上非阻塞方式处理消息
    pub fn DoMsgHandler(&self, request: &R) -> Option<R>
    where
        R: IRquest + Clone,
    {
        if let Some(handler) = self.Apis.get(&request.get_msgID()) {
            //执行对应处理方法
            handler.pre_handler(request);
            let r = handler.handler(request);
            handler.post_hander(request);
            Some(r)
        } else {
            println!("not exist");
            Some(request.clone())
        }
    }

    //为消息添加具体的处理逻辑
    pub fn AddRouter(&mut self, msgID: u32, router: Box<dyn IRouter<R = R> + Send + Sync>) {
        //1 判断当前msg绑定的API处理方法是否已经存在
        if self.Apis.contains_key(&msgID) {
            //执行对应处理方法
            panic!("repeated api , msgId =  {}", msgID)
        } else {
            self.Apis.insert(msgID, router);
        }
    }
}
