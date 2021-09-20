use std::sync::Arc;

use crate::ziface::{IRouter, IRquest};
use crate::znet::request::Request;
// use crate::znet::connection::ConnectionSync;

pub struct PingRouter;

// TODO 等声明周期完全熟悉后再考虑这个方法
// impl IRouter for PingRouter {
//     type R = Request<'a>;
//     fn pre_handler(&self, request: Request<'a>) {}
//     fn handler(&self, request: Request<'a>) {}
//     fn post_hander(&self, request: Request<'a>) {}
// }

impl IRouter for PingRouter {
    type R = Request;
    fn pre_handler(&self, request: &Request) {
        println!("pre {}", request.get_data());
    }
    fn handler(&self, request: &Request) -> Request {
        request.clone()
    }
    fn post_hander(&self, request: &Request) {
        println!("post {}", request.get_data());
    }
}

pub type RouterSync = Arc<Box<dyn IRouter<R = Request> + Send + Sync>>;

pub struct OneRouter;

impl IRouter for OneRouter {
    type R = Request;
    fn pre_handler(&self, request: &Request) {
        println!("ONE_PRE {}", request.get_data());
    }
    fn handler(&self, request: &Request) -> Request {
        let mut res = request.clone();
        res.data.reverse();
        res
    }
    fn post_hander(&self, request: &Request) {
        println!("ONE_POST {}", request.get_data());
    }
}

pub struct TwoRouter;

impl IRouter for TwoRouter {
    type R = Request;
    fn pre_handler(&self, request: &Request) {
        println!("TWO_PRE {}", request.get_data());
    }
    fn handler(&self, request: &Request) -> Request {
        let mut res = request.clone();
        res.data.reverse();
        res
    }
    fn post_hander(&self, request: &Request) {
        println!("TWO_POST {}", request.get_data());
    }
}
