use crate::message::RouterMessage;
use crate::RouterError;
use async_trait::async_trait;
use ockam::{Address, Context, Result, Worker};
use serde::{Deserialize, Serialize};

pub const ROUTER_ADDRESS: &str = "router";

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum RouteMessage {
    Route(RouterMessage, Option<Address>),
}

pub struct Router {}

#[async_trait]
impl Worker for Router {
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
            RouteMessage::Route(mut msg, address) => {
                if msg.onward_route.addrs.is_empty() {
                    return Err(RouterError::NoRoute.into());
                }
                let next_hop = msg.onward_route.addrs.remove(0);

                if let Some(a) = address {
                    msg.return_address(a.into());
                }
                ctx.send_message(next_hop.address, msg).await?;
                Ok(())
            }
        };
    }
}

#[cfg(test)]
mod test {
    use crate::message::{RouteableAddress, RouterMessage};
    use crate::router::{RouteMessage, Router};
    use async_trait::async_trait;
    use ockam::{Address, Result, Worker};

    pub struct MyWorker {
        pub address: String,
        pub router: String,
        pub first: bool,
        pub text: String,
        pub count: usize,
        pub is_first: bool,
    }

    #[async_trait]
    impl Worker for MyWorker {
        type Message = RouterMessage;
        type Context = ockam::Context;

        fn initialize(&mut self, _context: &mut Self::Context) -> Result<()> {
            self.count = 0;
            return Ok(());
        }

        fn shutdown(&mut self, _context: &mut Self::Context) -> Result<()> {
            Ok(())
        }

        async fn handle_message(
            &mut self,
            ctx: &mut Self::Context,
            mut msg: Self::Message,
        ) -> Result<()> {
            println!("{}", String::from_utf8(msg.payload.clone()).unwrap());
            if msg.onward_route.addrs.is_empty() {
                if self.count > 0 && self.is_first {
                    ctx.stop().await?;
                    return Ok(());
                } else {
                    msg.onward_route = msg.return_route.clone();
                    msg.return_route.addrs.truncate(0);
                }
            }
            let mut p = msg.payload.clone();
            p.append(&mut self.text.clone().as_bytes().to_vec());
            msg.payload = p;
            self.count += 1;
            let address: ockam::Address = self.address.clone().into();
            let r: Address = self.router.clone().into();
            ctx.send_message(r, RouteMessage::Route(msg, Some(address)))
                .await?;
            Ok(())
        }
    }

    #[test]
    fn route() {
        let (ctx, mut exe) = ockam::start_node();
        exe.execute(async move {
            let router = Router {};
            let w1 = MyWorker {
                address: String::from("w1"),
                router: String::from("router"),
                first: false,
                text: "1".to_string(),
                count: 0,
                is_first: true,
            };
            let w2 = MyWorker {
                address: String::from("w2"),
                router: String::from("router"),
                first: false,
                text: "2".to_string(),
                count: 0,
                is_first: false,
            };
            let w3 = MyWorker {
                address: String::from("w3"),
                router: String::from("router"),
                first: false,
                text: "3".to_string(),
                count: 0,
                is_first: false,
            };
            ctx.start_worker("router", router).await.unwrap();
            ctx.start_worker(w1.address.clone(), w1).await.unwrap();
            ctx.start_worker(w2.address.clone(), w2).await.unwrap();
            ctx.start_worker(w3.address.clone(), w3).await.unwrap();

            let mut m = RouterMessage::new();

            m.onward_address(RouteableAddress::Local(b"w1".to_vec()));
            m.onward_address(RouteableAddress::Local(b"w2".to_vec()));
            m.onward_address(RouteableAddress::Local(b"w3".to_vec()));
            m.payload = b"0".to_vec();

            ctx.send_message(String::from("router"), RouteMessage::Route(m, None))
                .await
                .unwrap();
        })
        .unwrap();
    }
}
