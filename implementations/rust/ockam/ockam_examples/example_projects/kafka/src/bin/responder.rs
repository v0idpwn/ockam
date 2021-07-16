use ockam::{stream::Stream, Context, Result, Route, SecureChannel, TcpTransport, Vault, TCP};
use kafka::Pong;
use std::time::Duration;

#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    // Create a pong worker
    ctx.start_worker("pong", Pong).await?;

    // Create a vault
    let vault = Vault::create(&ctx)?;

    // Create a secure channel listener at address "secure_channel_listener"
    SecureChannel::create_listener(&ctx, "secure_channel_listener", &vault).await?;

    // connect to hub on 4000
    let tcp = TcpTransport::create(&ctx).await?;
    tcp.connect("127.0.0.1:4000").await?;

    // Create a bi-directional stream
    Stream::new(&ctx)?
        .client_id("pong")
        .stream_service("stream_kafka")
        .index_service("stream_kafka_index")
        .with_interval(Duration::from_millis(100))
        .connect(
            Route::new().append_t(TCP, "127.0.0.1:4000"),
            // Stream name from THIS node to the OTHER node
            "pong_topic",
            // Stream name from OTHER to THIS
            "ping_topic",
        )
        .await?;

    // Don't call ctx.stop() here so this node runs forever.
    Ok(())
}
