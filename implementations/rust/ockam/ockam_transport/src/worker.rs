// use crate::traits::Connection;
// use ockam::{Context, Worker};
// use ockam_router::router::{RouteMessage, ROUTER_ADDRESS};
//
// use crate::traits::Connection;
// use async_trait::async_trait;
// use ockam::{Address, Context, Result, Worker};
// use ockam_router::message::{Route, RouterAddress, RouterMessage};
// use ockam_router::router::{RouteMessage, ROUTER_ADDRESS};
// use serde::{Deserialize, Serialize};
//
// #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
// pub enum ConnectionMessage {
//     SendMessage(RouterMessage),
//     ReceiveMessage,
// }
//
// #[async_trait]
// pub trait ConnectionWorker: Connection + Worker {}
//
// #[async_trait]
// impl Worker for dyn Connection {
//     type Message = ConnectionMessage;
//     type Context = Context;
//
//     fn initialize(&mut self, _context: &mut Self::Context) -> Result<()> {
//         Ok(())
//     }
//
//     fn shutdown(&mut self, _context: &mut Self::Context) -> Result<()> {
//         Ok(())
//     }
//
//     async fn handle_message(&mut self, ctx: &mut Self::Context, msg: Self::Message) -> Result<()> {
//         return match msg {
//             ConnectionMessage::SendMessage(m) => match self.send_message(m).await {
//                 Ok(_) => Ok(()),
//                 Err(e) => Err(e),
//             },
//             ConnectionMessage::ReceiveMessage => match self.receive_message().await {
//                 Ok(m) => ctx.send_message(
//                     ROUTER_ADDRESS.into(),
//                     RouteMessage::Route(m, Some(ctx.address())),
//                 ),
//                 Err(e) => Err(e),
//             },
//         };
//     }
// }
// //
// // #[cfg(test)]
// // mod tests {
// //     use tokio::runtime::Builder;
// //
// //     async fn run_connect_test(addr: String) {}
// //
// //     #[test]
// //     fn connect() {
// //         let runtime = Builder::new_current_thread()
// //             .enable_io()
// //             .enable_time()
// //             .build()
// //             .unwrap();
// //
// //         // runtime.block_on(async {
// //         //     run_connect_test(String::from("127.0.0.1:4052")).await;
// //         // });
// //     }
// // }
