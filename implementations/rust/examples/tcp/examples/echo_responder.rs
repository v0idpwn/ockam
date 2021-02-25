use ockam::Address;
use std::net::SocketAddr;
use std::str::FromStr;
use tcp_examples::echo::{Echo, EchoMessage};

fn main() {
    let (ctx, mut exe) = ockam::start_node();

    exe.execute(async move {
        //let listen_addr = SocketAddr::from_str("127.0.0.1:4050").unwrap();
        let listen_addr = SocketAddr::from_str("13.87.240.81:4000").unwrap();
        let mut listener = ockam_transport_tcp::TcpListener::create(listen_addr)
            .await
            .unwrap();
        let connection = listener.accept().await.unwrap();
        let echo = Echo {
            connection,
            count: 0,
        };

        let address: Address = "echo".into();
        ctx.start_worker(address, echo).await.unwrap();

        ctx.send_message("echo", EchoMessage::Receive)
            .await
            .unwrap();

        ctx.send_message("echo", EchoMessage::Send("hello".into()))
            .await
            .unwrap();
    })
    .unwrap();
}
