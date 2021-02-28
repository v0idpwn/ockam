use async_trait::async_trait;
use ockam::{Address, Context, Result, Worker};
use ockam_router::message::{RouteMessage, RouteableAddress, RouterAddress, RouterMessage};
use ockam_router::router::{Router, ROUTER_ADDRESS};
use ockam_transport::traits::Connection;
use ockam_transport_tcp::{
    ConnectionMessage, ConnectionWorkerTrait, ConnectionWorkerWrapper, TcpConnection,
};
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
                        "handle_message MainMsgReceiver, payload: {}",
                        String::from_utf8(m.payload.clone()).unwrap()
                    );
                    ctx.send_message(self.parent.clone(), MainWorkerMessage::Receive(m))
                        .await
                        .unwrap();
                    Ok(())
                } else {
                    // ToDo - error out
                    Ok(())
                }
            }
        };
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MainWorker {
    pub address: String,
    pub connection_worker: String, // connection worker address
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum MainWorkerMessage {
    Send(String),
    Receive(RouterMessage),
}

#[async_trait]
impl Worker for MainWorker {
    type Message = MainWorkerMessage;
    type Context = Context;

    fn initialize(&mut self, ctx: &mut Self::Context) -> Result<()> {
        Ok(())
    }

    fn shutdown(&mut self, _context: &mut Self::Context) -> Result<()> {
        Ok(())
    }

    async fn handle_message(&mut self, ctx: &mut Self::Context, msg: Self::Message) -> Result<()> {
        return match msg {
            MainWorkerMessage::Send(text) => {
                // create and start the message receiver service
                let msg_receiver = MainMessageReceiver {
                    parent: ctx.address().to_string(),
                };
                ctx.start_worker("echo_initiator_receiver", msg_receiver)
                    .await
                    .unwrap();

                // set up onward and return routes
                let mut m = RouterMessage::new();
                m.onward_address(RouteableAddress::Local(
                    self.connection_worker.as_bytes().to_vec(),
                ));
                m.onward_address(RouteableAddress::Local(b"echo_service".to_vec()));
                m.return_address(RouteableAddress::Local(ctx.address().to_vec()));
                m.payload = text.as_bytes().to_vec();

                ctx.send_message(self.connection_worker.clone(), ConnectionMessage::Alive)
                    .await
                    .unwrap();

                println!("initiator sending message");
                ctx.send_message(ROUTER_ADDRESS, RouteMessage::Send(m))
                    .await
                    .unwrap();

                ctx.send_message(self.connection_worker.clone(), ConnectionMessage::Alive)
                    .await
                    .unwrap();

                Ok(())
            }

            MainWorkerMessage::Receive(m) => {
                println!(
                    "main_service received {}",
                    String::from_utf8(m.payload).unwrap()
                );
                ctx.stop();
                Ok(())
            }
        };
    }
}

fn main() {
    let (mut ctx, mut exe) = ockam::start_node();

    exe.execute(async move {
        // create and start the router
        let r = Router {};
        ctx.start_worker(ROUTER_ADDRESS, r).await.unwrap();

        // create and start the connection worker
        let mut c = TcpConnection::create(SocketAddr::from_str("127.0.0.1:4050").unwrap());
        let connection_worker = c.get_worker_address();
        println!("connection_worker address: {:?}", connection_worker);
        ctx.start_worker(connection_worker.clone(), c)
            .await
            .unwrap();
        ctx.send_message(connection_worker.clone(), ConnectionMessage::Connect)
            .await
            .unwrap();

        // create and start the main worker
        let main_worker = MainWorker {
            address: String::from("echo_main"),
            connection_worker,
        };

        // tell the main worker to send a message
        let main_addr = main_worker.address.clone();
        ctx.start_worker(main_addr.clone(), main_worker)
            .await
            .unwrap();
        ctx.send_message(main_addr, MainWorkerMessage::Send("hello".into()))
            .await
            .unwrap();
    })
    .unwrap();
}
