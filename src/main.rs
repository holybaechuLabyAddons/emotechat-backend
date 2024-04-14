use emotechat_backend::database::mongo;
use dotenvy::dotenv;
use log::info;
use env_logger::Env;
use axum::{extract::Request, routing::get, Extension, Json, Router, ServiceExt};
use emotechat_backend::routes;
use serde_json::json;
use tower_http::normalize_path::NormalizePathLayer;
use tower_layer::Layer;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Connecting to MongoDB instance...");
    let db = mongo::connect().await.expect("Failed to connect to MongoDB instance");

    info!("Starting HTTP Server on {}:{}...", dotenvy::var("HOST").expect("HOST must be set"), dotenvy::var("PORT").expect("PORT must be set"));

    let router = Router::new()
        .route("/", get(|| async { Json(json!({
            "status": "ok",
        }))}))
        .merge(routes::v1::config())
        .layer(Extension(db.clone()));

    let app = ServiceExt::<Request>::into_make_service(
        NormalizePathLayer::trim_trailing_slash().layer(router)
    );

    let listener = tokio::net::TcpListener::bind(
        format!("{}:{}", dotenvy::var("HOST").unwrap(), dotenvy::var("PORT").unwrap())
    ).await.expect(format!("Failed to bind to {:?} port {:?}", dotenvy::var("HOST").unwrap(), dotenvy::var("PORT").unwrap()).as_str());
    
    axum::serve(listener, app).await.expect("Failed to start server");

    info!("Server started on http://{}:{}", dotenvy::var("HOST").unwrap(), dotenvy::var("PORT").unwrap());
}