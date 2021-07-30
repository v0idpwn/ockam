/// This example uses the stream service to send messages between two
/// clients.  A stream is a buffered message sending channel, which
/// means that you can run `initiator` and `responder` in any order
/// you like.
use ockam::{route, stream::Stream, Context, Result, TcpTransport, TCP};

#[ockam::node]
async fn main(mut ctx: Context) -> Result<()> {
    let _tcp = TcpTransport::create(&ctx).await?;

    // Set the address of the Kafka node you created here. (e.g. "192.0.2.1:4000")
    let hub_node_tcp_address = "<Your node Address copied from hub.ockam.network>";

    // Get stream names
    let sender_name = std::env::var("SENDER").unwrap();
    let receiver_name = std::env::var("RECEIVER").unwrap();

    // Create a stream client
    let (sender, _receiver) = Stream::new(&ctx)?
        .stream_service("stream_kafka")
        .index_service("stream_kafka_index")
        .client_id("stream-over-cloud-node-initiator")
        .connect(
            route![(TCP, hub_node_tcp_address)],
            sender_name,
            receiver_name,
        )
        .await?;

    // Send a message
    ctx.send(
        route![
            sender.clone(), // via the "initiator-to-responder" stream
            "echoer"        // to the "echoer" worker
        ],

    ctx.send(
        sender.to_route().append("echoer"),
        "Hello World!".to_string(),
    )
    .await?;

    // Receive a message from the "responder-to-initiator" stream
    let reply = ctx.receive_block::<String>().await?;
    println!("Reply via stream: {}", reply);

    ctx.stop().await
}
