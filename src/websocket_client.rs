use serde_json::{json, Value};
use std::error::Error;
use tokio_tungstenite::{client_async, tungstenite::protocol::Message};
use tokio_native_tls::native_tls::TlsConnector;
use tokio_native_tls::TlsStream;
use url::Url;
use futures_util::{sink::SinkExt, stream::StreamExt};
use hyper::http;
use prost::Message as ProstMessage;
use crate::config;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OpcUaData {
    #[prost(string, tag="1")]
    pub item: String,
    #[prost(float, tag="2")]
    pub value: f32,
    #[prost(int64, tag="3")]
    pub timestamp: i64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HistoricalValue {
    #[prost(int64, tag="1")]
    pub t: i64,
    #[prost(double, tag="2")]
    pub v: f64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Historical {
    #[prost(int64, tag="1")]
    pub batchid: i64,
    #[prost(string, tag="2")]
    pub sensor: String,
    #[prost(message, repeated, tag="3")]
    pub values: Vec<HistoricalValue>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Universal {
    #[prost(int32, repeated, tag="1")]
    pub type_: Vec<i32>,
    #[prost(bytes, repeated, tag="2")]
    pub messages: Vec<Vec<u8>>,
}

pub struct WebSocketClient;

impl WebSocketClient {
    pub async fn open_tls_stream(ws_url: &Url) -> Result<TlsStream<tokio::net::TcpStream>, Box<dyn Error>> {
        let connector = TlsConnector::builder().build()?;
        let connector: tokio_native_tls::TlsConnector = connector.into();

        let addrs = ws_url.socket_addrs(|| None)?;
        let stream = tokio::net::TcpStream::connect(&*addrs).await?;
        let tls_stream = connector.connect(ws_url.host_str().unwrap(), stream).await?;
        Ok(tls_stream)
    }

    pub fn create_request(url: &str) -> Result<http::Request<()>, Box<dyn Error>> {
        let parsed_url = Url::parse(url)?;
        let host = parsed_url.host_str().ok_or("Missing host")?;

        let authorization = config::get_authorization_header();

        let request = http::Request::builder()
            .method("GET")
            .uri(url)
            .header("Host", host)
            .header("Authorization", authorization)
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Key", tokio_tungstenite::tungstenite::handshake::client::generate_key())
            .header("Sec-WebSocket-Version", "13")
            .body(())?;

        Ok(request)
    }

    pub async fn run() -> Result<(Vec<String>, tokio_tungstenite::WebSocketStream<TlsStream<tokio::net::TcpStream>>), Box<dyn Error>> {
        let url = config::get_encoded_websocket_url();
        let ws_url = Url::parse(&url)?;

        let request = Self::create_request(&url)?;
        let tls = Self::open_tls_stream(&ws_url).await?;
        let (mut ws_stream, _) = client_async(request, tls).await?;
        println!("Connected to the WebSocket server");

        let initial_message = json!({
            "msg_type": "GET_TAGS",
        });

        ws_stream.send(Message::Text(initial_message.to_string())).await?;

        let mut valid_tags: Vec<String> = Vec::new();

        while let Some(msg_result) = ws_stream.next().await {
            match msg_result {
                Ok(msg) => {
                    match msg {
                        Message::Text(text) => {
                            match serde_json::from_str::<Value>(&text) {
                                Ok(parsed) => {
                                    if let Some(tags_array) = parsed.get("tags").and_then(|x| x.as_array()) {
                                        for tag in tags_array {
                                            if let Some(tag_value) = tag.get(0).and_then(Value::as_str) {
                                                valid_tags.push(tag_value.to_string());
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("Error parsing JSON: {}", e);
                                }
                            }
                        }
                        Message::Binary(_) => {}
                        Message::Ping(data) => {
                            ws_stream.send(Message::Pong(data)).await?;
                        }
                        Message::Close(_) => break,
                        _ => {}
                    }
                }
                Err(e) => {
                    println!("Error receiving message: {}", e);
                    break;
                }
            }

            if !valid_tags.is_empty() {
                break;
            }
        }

        Ok((valid_tags, ws_stream))
    }

    pub async fn send_protobuf_data(ws_stream: &mut tokio_tungstenite::WebSocketStream<TlsStream<tokio::net::TcpStream>>, universal: Universal) -> Result<(), Box<dyn Error>> {
        let mut universal_buf = Vec::new();
        universal.encode(&mut universal_buf)?;
      
        ws_stream.send(Message::Binary(universal_buf)).await?;
        Ok(())
    }
}