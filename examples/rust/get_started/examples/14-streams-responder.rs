use ockam::{route, stream::Stream, Context, Result, TcpTransport, TCP};
use ockam_get_started::Echoer;
use uuid::Uuid;

#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    let _tcp = TcpTransport::create(&ctx).await?;

    // Set the address of the Kafka node you created here. (e.g. "192.0.2.1:4000")
    let hub_node_tcp_address = "<Your node Address copied from hub.ockam.network>";

    // Generate unique stream names
    let sender_name = format!("responder-to-initiator:{}", Uuid::new_v4());
    let receiver_name = format!("initiator-to-receiver:{}", Uuid::new_v4());

    println!("\nRun the initiator example with the following environment variables:\n");
    println!("    SENDER={} \\", receiver_name);
    println!("    RECEIVER={} \\", sender_name);
    println!("    cargo run --example 14-streams-initiator\n");

    // Create a stream client
    Stream::new(&ctx)?
        .stream_service("stream_kafka")
        .index_service("stream_kafka_index")
        .client_id("stream-over-cloud-node-initiator")
        .connect(
            route![(TCP, hub_node_tcp_address)],
            sender_name,
            receiver_name,
        )
        .await?;

    // Start an echoer worker
    ctx.start_worker("echoer", Echoer).await?;

    // Don't call ctx.stop() here so this node runs forever.
    Ok(())
}
