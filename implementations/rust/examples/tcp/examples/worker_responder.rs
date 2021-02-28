use async_trait::async_trait;
use ockam::{Address, Context, Result, Worker};
use ockam_router::message::{RouteMessage, RouteableAddress, RouterAddress, RouterMessage};
use ockam_router::router::{Router, ROUTER_ADDRESS};
use ockam_transport::traits::Connection;
use ockam_transport_tcp::{ConnectionMessage, ConnectionWorkerTrait, TcpConnection, TcpListener};
use serde::{Deserialize, Serialize};
use serde_bare::error::Error::Message;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::task;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MainMessageReceiver {
    pub parent: String, // parent address
}

#[async_trait]
impl Worker for MainMessageReceiver {
    type Message = RouteMessage;
    type Context = Context;

    fn initialize(&mut self, _context: &mut Self::Context) -> Result<()> {
        Ok(())
    }

    fn shutdown(&mut self, _context: &mut Self::Context) -> Result<()> {
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Self::Context, msg: Self::Message) -> Result<()> {
        return match msg {
            RouteMessage::Send(_) => Ok(()),
            RouteMessage::Receive(m) => {
                if let Some(m) = m {
                    println!(
                        "handle_message EchoMsgReceiver, payload: {}",
                        String::from_utf8(m.payload.clone()).unwrap()
                    );
                    ctx.send_message(self.parent.clone(), EchoWorkerMessage::Receive(m))
                        .await
                        .unwrap();
                    Ok(())
                } else {
                    // ToDo error out
                    Ok(())
                }
            }
        };
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct EchoWorker {
    pub address: String,
    pub connection_worker: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum EchoWorkerMessage {
    Run,
    Receive(RouterMessage),
}

#[async_trait]
impl Worker for EchoWorker {
    type Message = EchoWorkerMessage;
    type Context = Context;

    fn initialize(&mut self, ctx: &mut Self::Context) -> Result<()> {
        Ok(())
    }

    fn shutdown(&mut self, _context: &mut Self::Context) -> Result<()> {
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Self::Context, msg: Self::Message) -> Result<()> {
        return match msg {
            EchoWorkerMessage::Run => {
                // create and start the message receiver service
                let msg_receiver = MainMessageReceiver {
                    parent: ctx.address().to_string(),
                };
                ctx.start_worker("echo_service", msg_receiver)
                    .await
                    .unwrap();
                Ok(())
            }
            EchoWorkerMessage::Receive(msg_received) => {
                println!(
                    "echo_service_main received {}",
                    String::from_utf8(msg_received.payload.clone()).unwrap()
                );
                let mut msg_to_send = RouterMessage::new();
                msg_to_send.onward_route = msg_received.return_route.clone();
                msg_to_send.payload = msg_received.payload.clone();
                ctx.send_message(
                    self.connection_worker.clone(),
                    RouteMessage::Send(msg_to_send),
                )
                .await
                .unwrap();
                ctx.stop();
                Ok(())
            }
        };
    }
}

fn main() {
    let (mut ctx, mut exe) = ockam::start_node();

    exe.execute(async move {
        // create and start router
        let r = Router {};
        ctx.start_worker(ROUTER_ADDRESS, r).await.unwrap();

        // listen for connection
        let mut l = TcpListener::create(SocketAddr::from_str("127.0.0.1:4050").unwrap())
            .await
            .unwrap();
        let c = l.accept().await.unwrap();

        // create and start connection worker
        let c_addr = c.get_worker_address();
        ctx.start_worker(c_addr.clone(), c).await.unwrap();

        // create and start main worker
        let echo_worker = EchoWorker {
            address: String::from("echo_service_main"),
            connection_worker: c_addr,
        };
    })
    .unwrap();
}
