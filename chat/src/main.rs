// Egui - Main

use anyhow::Result;
use async_broadcast::Sender;
use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    routing::{get, post},
    Router,
};
use std::env;
use tower_http::cors::CorsLayer;

mod routes;

#[derive(Debug, Clone)]
pub struct AppState {
    pub tx: Sender<String>,
    pub host: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Env
    dotenvy::dotenv()?;

    let host = env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or("8000".to_string());
    let (tx, _rx) = async_broadcast::broadcast::<String>(50);
    let host = format!("{}:{}", host, port);
    let site_url = format!("http://{}", &host);
    let cors = CorsLayer::new()
        .allow_origin(site_url.parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);
    let app = Router::new()
        .route("/", get(routes::home))
        .route("/sse", get(routes::message))
        .route("/create", post(routes::create_message))
        .with_state(AppState {
            tx,
            host: host.clone(),
        })
        .layer(cors);

    println!("Server starting in port {}!", host);
    let listener = tokio::net::TcpListener::bind(host).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
