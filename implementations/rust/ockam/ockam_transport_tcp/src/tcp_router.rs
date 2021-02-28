use async_trait::async_trait;
use hashbrown::HashMap;
use ockam::{Context, Worker};
use ockam_core::Result;
use ockam_router::router::RouteTransportMessage;

pub const TCP_ROUTER_ADDRESS: &str = "tcp_router";

use crate::{TcpConnection, TransportError};
use ockam_router::RouterError;

pub struct TcpMessageRouter {
    registry: HashMap<Vec<u8>, Box<TcpConnection>>, // <vectorized sockeaddr, worker address>,
}

impl TcpMessageRouter {
    pub fn new() -> Self {
        TcpMessageRouter {
            registry: HashMap::new(),
        }
    }
    pub fn register(&mut self, connection: Box<TcpConnection>) -> Result<()> {
        let key = connection.get_routeable_address();
        println!("tcp_router registered key: {:?}", key.clone(),);
        if self.registry.contains_key(&key) {
            return Err(RouterError::KeyInUse.into());
        }
        println!("...");
        if self.registry.insert(key, connection).is_some() {
            return Err(RouterError::Stop.into());
        }
        println!("Registered");
        Ok(())
    }
}

#[async_trait]
impl Worker for TcpMessageRouter {
    type Message = RouteTransportMessage;
    type Context = Context;

    fn initialize(&mut self, ctx: &mut Self::Context) -> Result<()> {
        println!("{} is running", ctx.address());
        Ok(())
    }

    fn shutdown(&mut self, _context: &mut Self::Context) -> Result<()> {
        Ok(())
    }

    async fn handle_message(&mut self, _ctx: &mut Self::Context, msg: Self::Message) -> Result<()> {
        println!("tcp_router got message");
        return match msg {
            RouteTransportMessage::Route(mut msg) => {
                let tcp_addr = msg.onward_route.addrs.remove(0);
                let connection = self.registry.remove(&tcp_addr.address);
                if connection.is_none() {
                    println!("None!!!");
                    return Err(RouterError::NoSuchKey.into());
                }
                let mut connection = connection.unwrap();
                if connection.send_message(msg).await.is_err() {
                    return Err(TransportError::ConnectionClosed.into());
                }
                if connection.receive_message().await.is_err() {
                    return Err(TransportError::ConnectionClosed.into());
                }
                self.registry
                    .insert(connection.get_routeable_address(), connection);
                Ok(())
            }
        };
    }
}
