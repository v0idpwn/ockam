//! # Remote Mailbox/ Stream service
//!
//! A stream proxy in the cloud.

use ockam_core::ProtocolId;
use serde::{Deserialize, Serialize};

/*

{
  "stream_type": "CreateStreamRequest",
  "data": [....],
}

StreamPull (req/resp)
StreamPush (req/resp)
StreamCreate (req/resp)
Index (req/resp)

Error

*/

pub fn parse<T>(id: ProtocolId, payload: Vec<u8>) -> T {
    todo!()
}

/// Basic protocol definitions for this service
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamProtocol {
    /// Request a new mailbox to be created
    CreateStreamRequest { stream_name: Option<String> },
    /// Response to a `CreateStreamRequest`
    Init { stream_name: String },
    /// Push a message into the mailbox
    PushRequest { request_id: usize, data: Vec<u8> },
    /// Confirm push operation on the mailbox
    PushConfirm {
        request_id: usize,
        status: Option<()>,
    },
    /// Pull messages from the mailbox
    PullRequest {
        request_id: usize,
        index: usize,
        limit: usize,
    },
    /// Response to a `PullRequest`
    PullResponse {
        request_id: usize,
        messages: Vec<StreamMessage>,
    },
}

impl StreamProtocol {
    pub fn parse(vec: &Vec<u8>) -> Result<Self> {
        todo!()
    }
}

#[test]
fn stream_protocol() {
    let d = StreamProtocol::CreateStreamRequest {
        stream_name: Some("MyStream".to_string()),
    };

    let erray = vec![
        13, 115, 116, 114, 101, 97, 109, 95, 99, 114, 101, 97, 116, 101, 10,
    ];

    let estr = unsafe { std::str::from_utf8_unchecked(&erray) };
    println!(">>> {} <<<", estr);

    let data = serde_bare::to_vec(&d).unwrap();
    println!("{:?}", data);
}

pub enum IndexProtocol {
    /// Get the index for a particular mailbox
    GetIndex {
        stream_name: String,
        client_id: String,
    },
    /// Reply to a `GetIndex`
    Index {
        stream_name: String,
        client_id: String,
        index: usize,
    },
    /// Update the index
    SaveIndex {
        stream_name: String,
        client_id: String,
        index: usize,
    },
}

/// A stream message with a mailbox index
#[derive(Serialize, Deserialize)]
pub struct StreamMessage {
    /// Index of the message in the stream
    pub index: usize,
    /// Encoded data of the message
    pub data: Vec<u8>,
}
