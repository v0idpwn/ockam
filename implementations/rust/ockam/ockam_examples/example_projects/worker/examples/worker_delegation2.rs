use ockam::{mailbox::StreamProtocol, Address, Any, Context, ProtocolId, Result, Routed, Worker};

#[derive(Default)]
struct ProxyWorker {
    mailbox: Option<Address>,
    stream_name: Option<String>,
}

/// A protocol that is spoken by the worker to itself
enum WorkerProtocol {
    Init(Address, String),
    Error,
}

#[ockam::async_worker]
impl Worker for ProxyWorker {
    type Context = Context;
    type Message = Any;

    async fn initialize(&mut self, ctx: &mut Context) -> Result<()> {

        // Register a handle for the protocol "stream"
        // 
        // Actually these functions can't be generic in `Context`. Either
        // we: add another associative type to Worker, or pull these
        // functions out of Context, to be held by the worker data
        // structure directly (really more of a closure mapping utility).
        ctx.register_protocol("stream", |msg, ctx| -> WorkerProtocol {
            let sender = msg.return_route.sender();
            let msg = StreamProtocol::parse(&msg.payload).unwrap();
            match msg {
                StreamProtocol::Init { stream_name } => {
                    WorkerProtocol::Init(sender, stream_name)
                },
                _ => todo!()
            }
        });

        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, msg: Routed<Any>) -> Result<()> {
        match ctx.handle_protocol(msg.protocol(), msg) {
            // Apply state changes based on what the protocol parser returns
            WorkerProtocol::Init(addr, name) => {
                self.stream_name = Some(name);
                self.mailbox = Some(addr);
            },
            // For example this is an error
            WorkerProtocol::Error => {
                error!("Couldn't parse incoming protocol message");
            }
        }

        Ok(())
    }
}


#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    ctx.start_worker("proxy", ProxyWorker::default()).await?;

    // Send a message to "proxy" with the protocol "stream"
    ctx.send_protocol(
        "proxy",
        "stream",
        StreamProtocol::Init {
            stream_name: "your-stream".to_string(),
        },
    )
    .await?;

    Ok(())
}
