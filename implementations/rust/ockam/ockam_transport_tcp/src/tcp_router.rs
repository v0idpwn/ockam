use async_trait::async_trait;
use hashbrown::HashMap;
use ockam::{Context, Worker, Address};
use ockam_core::Result;
use ockam_router::router::RouteTransportMessage;
use crate::{TransportError, TcpWorkerMessage};
use ockam_router::{RouterError, RouterAddress};


pub const TCP_ROUTER_ADDRESS: &str = "tcp_router";

pub struct TcpMessageRouter {
    registry: HashMap<Vec<u8>, Address>, // <vectorized sockeaddr, worker address>,
}

impl TcpMessageRouter {
    pub fn new() -> Self {
        TcpMessageRouter {
            registry: HashMap::new(),
        }
    }
    pub fn register(&mut self, addr: Address) -> Result<()> {
        println!("--------tcp_router registered key: {:?}", &addr,);
        if self.registry.contains_key(&addr.to_vec()) {
            return Err(RouterError::KeyInUse.into());
        }
        if self.registry.insert(addr.to_vec(), addr).is_some() {
            return Err(RouterError::Stop.into());
        }
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

    async fn handle_message(&mut self, ctx: &mut Self::Context, msg: Self::Message) -> Result<()> {
        println!("tcp_router got message");
        return match msg {
            RouteTransportMessage::Route(mut msg) => {
                let tcp_addr = msg.onward_route.addrs.remove(0);
                let key = serde_bare::to_vec::<RouterAddress>(&tcp_addr).unwrap();
                let addr = self.registry.get(&key);
                println!("tcp_router looking up {:?}", key);
                if addr.is_none() {
                    println!("------no such key {:?}",key);
                    return Err(RouterError::NoSuchKey.into());
                }
                let addr = addr.unwrap().clone();
                println!("tcp_router sending message to worker {:?}", addr.clone());
                if ctx.send_message(addr.clone(), TcpWorkerMessage::SendMessage(msg)).await.is_err() {
                    return Err(TransportError::ConnectionClosed.into());
                }
                println!("tcp_router message sent, now receiving");
                if ctx.send_message(addr, TcpWorkerMessage::Receive).await.is_err() {
                    return Err(TransportError::ConnectionClosed.into());
                }
                println!("tcp_router sent receive");
                Ok(())
            },
            _ => Ok(())
        };
    }
}
