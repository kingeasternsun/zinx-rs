use zinx_rs::znet_async::Server;
#[tokio::main]
async fn main() {
    let mut ser = Server::new(
        String::from("name"),
        String::from("ipv4"),
        String::from("127.0.0.1"),
        9090,
    );
    ser.Serve().await;
    ser.Stop().await
}
