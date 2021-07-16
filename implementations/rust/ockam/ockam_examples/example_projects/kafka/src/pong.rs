use ockam::{Context, Result, Routed, Worker};

pub struct Pong;

#[ockam::worker]
impl Worker for Pong {
    type Context = Context;
    type Message = String;

    async fn handle_message(&mut self, ctx: &mut Context, message: Routed<String>) -> Result<()> {
        println!("\nPong worker received ping for: {}",
                 message);
        println!("Pong worker sending pong: {}", message.return_route());

        // Echo the message body back on its return_route.
        ctx.send(message.return_route(), message.body()).await?;

        Ok(())
    }
}
