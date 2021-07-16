use ockam::{stream::Stream, Address, Context, Message, Result, Route, SecureChannel, TcpTransport, Vault, TCP};
use ockam_core::{LocalMessage, TransportMessage};
use kafka::Ping;
use std::time::Duration;

#[ockam::node]
async fn main(mut ctx: Context) -> Result<()> {
    // Create a ping worker
    ctx.start_worker("ping", Ping::default()).await?;

    // Create a vault
    let vault = Vault::create(&ctx)?;

    // connect to hub on 4000
    let tcp = TcpTransport::create(&ctx).await?;
    tcp.connect("127.0.0.1:4000").await?;

    // Create a bi-directional stream
    let (stream_tx, _) = Stream::new(&ctx)?
        .client_id("ping")
        .stream_service("stream_kafka")
        .index_service("stream_kafka_index")
        .with_interval(Duration::from_millis(100))
        .connect(
            Route::new().append_t(TCP, "127.0.0.1:4000"),
            // Stream name from THIS node to the OTHER node
            "ping_topic",
            // Stream name from OTHER to THIS
            "pong_topic",
        )
        .await?;

    // Create a secure channel via the stream
    let secure_channel = SecureChannel::create(
        &ctx,
        Route::new()
            // Send via the stream
            .append(stream_tx.clone())
            // And then to the secure_channel_listener
            .append("secure_channel_listener"),
        &vault,
    )
    .await?;

    let transport_message = TransportMessage::v1(
        Route::new().append(secure_channel.address()).append("pong"),
        Route::new().append("ping"),
        "0".to_string().encode().unwrap(),
    );

    let local_message = LocalMessage::new(transport_message, vec![]);
    ctx.forward(local_message).await?;

    /*ctx.send_from_address(
        Route::new().append(secure_channel.address()).append("pong"),
        "0".to_string(),
        ctx.address() // Address::from_string("0#ping")
    )
    .await?;*/

    // Don't call ctx.stop() here so this node runs forever.
    Ok(())
}
