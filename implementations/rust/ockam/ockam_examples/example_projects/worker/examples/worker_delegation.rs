use ockam::{mailbox::StreamProtocol, Address, Any, Context, ProtocolId, Result, Routed, Worker};

#[derive(Default)]
struct ProxyWorker {
    mailbox: Option<Address>,
    stream_name: Option<String>,
}

#[ockam::async_worker]
impl Worker for ProxyWorker {
    type Context = Context;
    type Message = Any;

    async fn initialize(&mut self, ctx: &mut Context) -> Result<()> {
        // ctx.delegate("mailbox", MailboxWorker).await?;

        // With a "delegate" call this could be done internally
        ctx.start_worker("child-stream", MailboxWorker).await?;

        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Context, msg: Routed<Any>) -> Result<()> {
        match msg.protocol().as_str() {
            "stream" => {
                let mut msg = msg.into_transport_message();
                println!("Proxy 'stream' protocol to child worker: {:?}", msg.payload);
                msg.onward_route.modify().prepend("child-stream");
                ctx.forward(msg).await?;
            }
            proto => println!("Unknown protocol '{}'", proto),
        }

        Ok(())
    }
}

struct MailboxWorker;

#[ockam::async_worker]
impl Worker for MailboxWorker {
    type Context = Context;
    type Message = StreamProtocol;

    async fn handle_message(
        &mut self,
        ctx: &mut Context,
        msg: Routed<StreamProtocol>,
    ) -> Result<()> {
        match &*msg {
            StreamProtocol::Init { stream_name } => {
                println!("Handling Stream '{}' INIT event!", stream_name);
            }
            _ => todo!(),
        }

        ctx.stop().await
    }
}

#[ockam::node]
async fn main(ctx: Context) -> Result<()> {
    ctx.start_worker("proxy", ProxyWorker::default()).await?;

    assert_eq!(
        ProtocolId::from("stream").as_str().to_ascii_lowercase(),
        "stream".to_string()
    );

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

// This is an experiment as to how a child worker could influence the
// parent worker state.  Define a `WorkerProxy` trait which specifies
// a new protocol, which is then implemented by the parent worker.
// Child workers can then update the parent worker state.
//
// 2 Problems with this approach:
//
// - Child workers can't _get_ current state from the worker because
//   this would involve them having to delegate this new protocol to
//   another child worker...
// - A lot of overhead in creating this protocol.  Maybe a special
//   macro could fix that, but it's still a lot of new code.

/// P is a user-defined type that maps actions to the internal state
/// of the worker.  This is a protocol of its own
#[ockam::async_worker]
pub trait WorkerProxy<P> {
    type Context;

    /// Implement this function to apply state requests and changes from child workers
    async fn apply(&mut self, _: &mut Self::Context, _: P) -> Result<()>;
}

pub enum MailboxStateProtocol {
    Mailbox(String, Address),
}

#[ockam::async_worker]
impl WorkerProxy<MailboxStateProtocol> for ProxyWorker {
    type Context = Context;

    async fn apply(&mut self, _: &mut Context, p: MailboxStateProtocol) -> Result<()> {
        match p {
            MailboxStateProtocol::Mailbox(name, addr) => {
                self.stream_name = Some(name);
                self.mailbox = Some(addr);
            }
        }

        Ok(())
    }
}
