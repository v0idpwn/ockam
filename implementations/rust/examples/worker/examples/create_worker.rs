use ockam::{Context, Handler, Message, Result, Worker};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

struct Printer;

#[derive(Debug, Serialize, Deserialize)]
struct PrintMessage(String);

impl Message for PrintMessage {}

#[async_trait]
impl Worker for Printer {
    type Context = Context;

    fn initialize(&mut self, _context: &mut Self::Context) -> Result<()> {
        println!("Printer starting");
        Ok(())
    }
}

#[async_trait]
impl Handler<PrintMessage> for Printer {
    async fn handle(&mut self, msg: PrintMessage, _ctx: &mut Context) {
        println!("PRINTER: {}", msg.0);
    }
}

fn main() {
    let (ctx, mut exe) = ockam::node();

    exe.execute(async move {
        let node = ctx.node();

        node.start_worker("printer", Printer {}).await.unwrap();

        node.send_message::<Printer, _, _>(
            "printer",
            PrintMessage {
                0: "Hello printer worker".to_string(),
            },
        )
        .await
        .unwrap();

        // RACE CONDITION: Because this call is so soon after,
        // sometimes the worker is killed before the msg reaches it
        node.stop().await.unwrap();
    })
    .unwrap();
}
