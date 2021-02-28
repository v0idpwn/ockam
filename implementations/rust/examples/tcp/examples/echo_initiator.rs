use async_trait::async_trait;
use ockam::{Address, Context, Result, Worker};
use ockam_router::{
    LocalRouter, RouteTransportMessage, RouteableAddress, Router, TransportMessage,
    LOCAL_ROUTER_ADDRESS, ROUTER_ADDRESS, ROUTER_ADDRESS_TYPE_LOCAL, ROUTER_ADDRESS_TYPE_TCP,
};
use ockam_transport_tcp::{TcpConnection, TcpMessageRouter, TCP_ROUTER_ADDRESS};
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::time::{sleep, Duration};

pub struct InitiatorEchoRelay {}

impl InitiatorEchoRelay {
    pub fn new() -> Self {
        InitiatorEchoRelay {}
    }
}

#[async_trait]
impl Worker for InitiatorEchoRelay {
    type Message = RouteTransportMessage;
    type Context = Context;

    fn initialize(&mut self, _ctx: &mut Self::Context) -> Result<()> {
        Ok(())
    }

    fn shutdown(&mut self, _context: &mut Self::Context) -> Result<()> {
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Self::Context, msg: Self::Message) -> Result<()> {
        return match msg {
            RouteTransportMessage::Route(m) => {
                println!(
                    "initiator received message: {}",
                    String::from_utf8(m.payload).unwrap()
                );
                ctx.stop().await.unwrap();
                Ok(())
            }
        };
    }
}

#[ockam::node]
async fn main(ctx: ockam::Context) {
    // create and register everything
    // main router
    let mut router = Router::new();

    // local router
    let mut local_router = LocalRouter::new();
    if let Err(e) = router.register(
        ROUTER_ADDRESS_TYPE_LOCAL,
        Address::from(LOCAL_ROUTER_ADDRESS),
    ) {
        println!("{:?}", e);
        ctx.stop().await.unwrap();
    }

    // tcp router
    let mut tcp_router = TcpMessageRouter::new();
    if let Err(e) = router.register(ROUTER_ADDRESS_TYPE_TCP, Address::from(TCP_ROUTER_ADDRESS)) {
        println!("{:?}", e);
        ctx.stop().await.unwrap();
    }

    // create and register the tcp connection
    let socket_addr = SocketAddr::from_str("127.0.0.1:4050").unwrap();
    let mut connection = TcpConnection::create(socket_addr.clone());
    if let Err(e) = connection.connect().await {
        ctx.stop().await.unwrap();
        println!("{:?}", e);
        return;
    }
    tcp_router.register(connection).unwrap();

    // create and register the echo message relay
    let relay = InitiatorEchoRelay::new();
    let echo_service_addr = Address::from("echo_service");
    local_router
        .register(Address::from("echo_service"))
        .unwrap();

    // start all the workers
    // main router
    ctx.start_worker(ROUTER_ADDRESS, router).await.unwrap();

    // tcp router
    ctx.start_worker(TCP_ROUTER_ADDRESS, tcp_router)
        .await
        .unwrap();

    // local router
    ctx.start_worker(LOCAL_ROUTER_ADDRESS, local_router)
        .await
        .unwrap();

    // relay worker
    ctx.start_worker(echo_service_addr.clone(), relay)
        .await
        .unwrap();

    // create and send the message
    sleep(Duration::from_millis(50)).await;
    let mut msg = TransportMessage::new();
    msg.onward_address(RouteableAddress::Tcp(socket_addr));
    msg.onward_address(RouteableAddress::Local(b"echo_service".to_vec()));
    msg.return_address(RouteableAddress::Local(b"echo_service".to_vec()));
    ctx.send_message(ROUTER_ADDRESS, RouteTransportMessage::Route(msg))
        .await
        .unwrap();
    println!("message sent");
}
