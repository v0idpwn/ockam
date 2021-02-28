use ockam::{self, Context, Result};
use ockam_router::router::RouterMessage::Route;
use ockam_router::router::{Router, RouterMessage, ROUTER_ADDRESS, ROUTER_ADDRESS_TYPE_TCP};
use ockam_transport_tcp::tcp_router::{TcpMessageRouter, TCP_ROUTER_ADDRESS};
use std::net::SocketAddr;
use std::str::FromStr;

#[ockam::node]
async fn main(ctx: ockam::Context) {
    // start the router
    let router = Router::new();
    ctx.start_worker(ROUTER_ADDRESS, router).await;

    // start the tcp router
    let tcp_router = TcpMessageRouter::new();
    ctx.start_worker(TCP_ROUTER_ADDRESS, tcp_router).await;

    // register the tcp router
    ctx.send_message(
        ROUTER_ADDRESS,
        RouterMessage::Register(ROUTER_ADDRESS_TYPE_TCP, TCP_ROUTER_ADDRESS.into()),
    )
    .await;

    // create the tcp transport
    let listen_addr = SocketAddr::from_str("127.0.0.1:4051").unwrap();
    let mut listener = ockam_transport_tcp::TcpListener::create(listen_addr)
        .await
        .unwrap();
    let connection = listener.accept().await.unwrap();
    println!("Listener Connected!!");
}
