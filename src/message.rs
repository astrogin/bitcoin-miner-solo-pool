use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
//different type of messages from pool
pub enum Message {
    StandardRequest(StandardRequest),
    Notification(Notification),
    OkResponse(Response),
    ErrorResponse(Response),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct StandardRequest {
    pub id: String,
    pub method: String,
    pub params: Value,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Notification {
    pub method: String,
    pub params: Value,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Response {
    pub id: String,
    pub error: Option<JsonRpcError>,
    pub result: Value,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Serialize, Deserialize)]
pub struct MinerDataMessage {
    pub method: String,
    pub params: Value,
    pub nonce1: String,
}

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub method: String,
    pub params: Vec<String>,
}

impl From<Response> for Message {
    fn from(res: Response) -> Self {
        if res.error.is_some() {
            Message::ErrorResponse(res)
        } else {
            Message::OkResponse(res)
        }
    }
}

impl From<StandardRequest> for Message {
    fn from(sr: StandardRequest) -> Self {
        Message::StandardRequest(sr)
    }
}

impl From<Notification> for Message {
    fn from(n: Notification) -> Self {
        Message::Notification(n)
    }
}
