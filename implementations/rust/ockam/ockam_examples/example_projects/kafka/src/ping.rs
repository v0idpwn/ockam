use ockam::{Context, Result, Routed, Worker};
use std::time::Duration;

struct State {
    last: usize
}

pub struct Ping {
    state: State,
}

impl Default for Ping {
    fn default() -> Self {
        Self {
            state: State {
                last: 0
            }
        }
    }
}

#[ockam::worker]
impl Worker for Ping {

    type Context = Context;
    type Message = String;

    async fn handle_message(&mut self, ctx: &mut Context, message: Routed<String>) -> Result<()> {
        let previous = message.clone().parse::<usize>().unwrap();
        println!("\nPing worker received pong for: {}", previous);

        self.state = match self.state.last {
            high if high > previous => {
                State {
                    last: high
                }
            },
            _low => {
                let next = previous + 1;

                ctx.sleep(Duration::from_millis(1000)).await;
                println!("Ping worker sending ping: {}", next);

                let reply = format!("{}", next);
                ctx.send(message.return_route(), reply).await?;

                State {
                    last: next
                }
            },
        };

        Ok(())
    }
}
