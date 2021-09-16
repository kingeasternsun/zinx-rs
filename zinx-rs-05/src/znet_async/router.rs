use std::sync::Arc;

use crate::ziface::{IRouter, IRquest};
use crate::znet_async::request::Request;

pub struct PingRouter;

impl IRouter for PingRouter {
    type R = Request;
    fn pre_handler(&self, request: Request) {
        println!("pre {}", request.get_data());
    }
    fn handler(&self, _request: Request) {}
    fn post_hander(&self, request: Request) {
        println!("post {}", request.get_data());
    }
}

pub type RouterSync = Arc<Box<dyn IRouter<R = Request> + Send + Sync>>;
