#![allow(non_snake_case)]
pub trait Iserver {
    //  启动服务器的方法
    fn Start(&mut self) -> std::io::Result<()>;
    // 停止服务器的方法
    fn Stop(&mut self);
    // 开启业务服务方法
    fn Serve(&mut self);
}
