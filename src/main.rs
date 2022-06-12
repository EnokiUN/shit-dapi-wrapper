use futures::{
    stream::{SplitSink, StreamExt},
    SinkExt,
};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, Value};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{net::TcpStream, sync::Mutex, task, time};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

const TOKEN: &str = "";
const API_URL: &str = "https://discord.com/api/v10/";
const GATEWAY_URL: &str = "wss://gateway.discord.gg/?v=9&encoding=json";

type Err = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Serialize, Deserialize)]
struct GatewayPayload {
    t: Option<String>,
    s: Option<i32>,
    op: i32,
    d: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SendableGatewayPayload {
    op: i32,
    d: GatewayPayloadData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum GatewayPayloadData {
    Identify(IdentifyData),
}

#[derive(Debug, Serialize, Deserialize)]
struct IdentifyProperties {
    #[serde(rename = "$os")]
    os: String,
    #[serde(rename = "$device")]
    browser: String,
    #[serde(rename = "$device")]
    device: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct IdentifyData {
    token: String,
    intents: u16,
    properties: IdentifyProperties,
}

struct Client {
    token: String,
}

impl Client {
    async fn start(self) -> Result<(), Err> {
        let (ws, _) = connect_async(GATEWAY_URL).await?;
        let (tx, mut rx) = ws.split();
        let tx = Arc::new(Mutex::new(tx));

        tx.lock()
            .await
            .send(Message::Text(
                json!({
                    "op": 2_u8,
                    "d": {
                        "token": self.token.to_string(),
                        "intents": 46703_u16,
                        "properties": {
                            "$os": "linux",
                            "$device": "sussysus",
                            "$platform": "sussysus",
                        }
                    }
                })
                .to_string(),
            ))
            .await?;

        while let Some(msg) = rx.next().await {
            if let Ok(Message::Text(msg)) = msg {
                task::spawn(Client::handle_payload(
                    tx.clone(),
                    from_str::<GatewayPayload>(&msg)?,
                ));
            }
        }

        Ok(())
    }
    async fn handle_payload(
        tx: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
        payload: GatewayPayload,
    ) -> Result<(), Err> {
        match payload.op {
            0 => match payload.t.unwrap().as_str() {
                "MESSAGE_CREATE" => {
                    let data = payload.d.unwrap();
                    if data["author"]["id"].as_str().unwrap() == "980529466478567495" {
                        return Ok(());
                    }
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert(
                        "Authorization",
                        ("Bot ".to_string() + TOKEN).parse().unwrap(),
                    );
                    reqwest::Client::new()
                        .post(
                            API_URL.to_string()
                                + &format!(
                                    "channels/{}/messages",
                                    data["channel_id"].as_str().unwrap()
                                ),
                        )
                        .headers(headers)
                        .json(&json!({"content": "shit"}))
                        .send()
                        .await?
                        .json()
                        .await?;
                }
                _ => {
                    println!("shit");
                }
            },
            10 => {
                if let Value::Number(interval) = &payload.d.unwrap()["heartbeat_interval"] {
                    loop {
                        tx.lock()
                            .await
                            .send(Message::Text(
                                json!({"op": 1_u8,
                            "d": Option::<u32>::None})
                                .to_string(),
                            ))
                            .await?;

                        time::sleep(Duration::from_millis(interval.as_u64().unwrap())).await;
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    if TOKEN.is_empty() {
        println!("You should probably specify a token lol");
        return;
    }
    Client {
        token: TOKEN.to_string(),
    }
    .start()
    .await
    .unwrap();
}
