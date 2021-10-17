use std::sync::Arc;

use crate::ziface::{IRouter, IRquest};
use crate::znet::request::Request;
// use crate::znet::connection::ConnectionSync;

pub struct PingRouter;

// TODO 等生命周期完全熟悉后再考虑这个方法
// impl IRouter for PingRouter {
//     type R = Request<'a>;
//     fn pre_handler(&self, request: Request<'a>) {}
//     fn handler(&self, request: Request<'a>) {}
//     fn post_hander(&self, request: Request<'a>) {}
// }

impl IRouter for PingRouter {
    type R = Request;
    fn pre_handler(&self, request: Request) {
        println!("pre {:?}", request.get_data());
    }
    fn handler(&self, request: Request) {}
    fn post_hander(&self, request: Request) {
        println!("post {:?}", request.get_data());
    }
}

pub type RouterSync = Arc<Box<dyn IRouter<R = Request> + Send + Sync>>;
