use ockam::Address;
use ockam_transport_tcp::TcpConnection;
use std::net::SocketAddr;
use std::str::FromStr;
use tcp_examples::echo::{Echo, EchoMessage};

fn main() {
    let (ctx, mut exe) = ockam::start_node();

    exe.execute(async move {
        let connect_addr = SocketAddr::from_str("13.87.240.81:4000").unwrap();
        let mut connection = TcpConnection::create(connect_addr);
        connection.connect().await.unwrap();

        let echo = Echo {
            connection,
            count: 0,
        };

        let address: Address = "echo".into();
        ctx.start_worker(address, echo).await.unwrap();

        ctx.send_message("echo", EchoMessage::Send("hello".into()))
            .await
            .unwrap();

        ctx.send_message("echo", EchoMessage::Receive)
            .await
            .unwrap();
    })
    .unwrap();
}
