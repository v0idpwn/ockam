use async_trait::async_trait;
use ockam::Context;
use ockam::Worker;
use ockam_router::message::{RouterAddress, RouterMessage, ROUTER_ADDRESS_LOCAL};
use ockam_transport_tcp::Connection;
use serde::{Deserialize, Serialize};

pub struct Echo {
    pub connection: Box<dyn Connection>,
    pub count: usize,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum EchoMessage {
    Send(Vec<u8>),
    Receive,
}

#[async_trait]
impl Worker for Echo {
    type Message = EchoMessage;
    type Context = Context;

    fn initialize(&mut self, _context: &mut Self::Context) -> ockam::Result<()> {
        Ok(())
    }

    fn shutdown(&mut self, _context: &mut Self::Context) -> ockam::Result<()> {
        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Self::Context,
        msg: Self::Message,
    ) -> ockam::Result<()> {
        return match (msg) {
            EchoMessage::Send(text) => {
                let mut msg_out = RouterMessage::new();
                msg_out.onward_route.addrs.push(RouterAddress {
                    address_type: ROUTER_ADDRESS_LOCAL,
                    address: b"echo_service".to_vec(),
                });
                msg_out.return_route.addrs.push(RouterAddress {
                    address_type: ROUTER_ADDRESS_LOCAL,
                    address: b"echo_service".to_vec(),
                });
                msg_out.payload = text;
                self.connection.send_message(msg_out).await?;
                println!("sent \"hello\"");
                self.count += 1;
                if self.count == 2 {
                    ctx.stop().await.unwrap();
                }
                Ok(())
            }
            EchoMessage::Receive => {
                let m = self.connection.receive_message().await?;
                println!("received \"{}\"", String::from_utf8(m.payload).unwrap());
                self.count += 1;
                if self.count == 2 {
                    ctx.stop().await.unwrap();
                }
                Ok(())
            }
        };
    }
}
