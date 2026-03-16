use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};

use super::handler::ConnectionState;
use super::messages::Request;

/// Start the AWP WebSocket server.
pub async fn start(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).await?;
    info!("AWP server listening on ws://{}", addr);

    loop {
        let (stream, peer) = listener.accept().await?;
        info!(%peer, "New connection");

        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(ws_stream) => {
                    handle_connection(ws_stream, peer).await;
                }
                Err(e) => {
                    error!(%peer, "WebSocket handshake failed: {}", e);
                }
            }
        });
    }
}

async fn handle_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    peer: std::net::SocketAddr,
) {
    let (mut sink, mut stream) = ws_stream.split();
    let mut state = ConnectionState::new();

    while let Some(msg) = stream.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let response = match serde_json::from_str::<Request>(&text) {
                    Ok(req) => {
                        super::handler::handle_request(
                            &req.id,
                            &req.method,
                            &req.params,
                            &mut state,
                        )
                        .await
                    }
                    Err(e) => {
                        warn!(%peer, "Invalid JSON: {}", e);
                        super::messages::Response::error(
                            "",
                            super::messages::ErrorCode::InvalidRequest,
                            &format!("Invalid JSON: {}", e),
                        )
                    }
                };

                let response_json = serde_json::to_string(&response).unwrap_or_else(|e| {
                    format!(
                        r#"{{"id":"","type":"response","error":{{"code":"INTERNAL","message":"Serialization error: {}"}}}}"#,
                        e
                    )
                });

                if let Err(e) = sink.send(Message::Text(response_json.into())).await {
                    error!(%peer, "Failed to send response: {}", e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                info!(%peer, "Connection closed");
                break;
            }
            Ok(Message::Ping(data)) => {
                if let Err(e) = sink.send(Message::Pong(data)).await {
                    error!(%peer, "Failed to send pong: {}", e);
                    break;
                }
            }
            Err(e) => {
                error!(%peer, "WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}
