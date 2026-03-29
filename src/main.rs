mod auth;
mod db;
mod models;
mod routes;

use axum::Router;
use routes::AppState;
use sqlx::sqlite::SqlitePoolOptions;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

async fn shutdown_signal() {
  tokio::signal::ctrl_c()
    .await
    .expect("failed to install Ctrl+C handler");
}

fn port() -> u16 {
  std::env::var("SKIPS_DB_PORT")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(3054)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let database_url = std::env::var("DATABASE_URL")
    .unwrap_or_else(|_| "sqlite:data/skips.db?mode=rwc".to_string());
  let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect(&database_url)
    .await?;
  
  sqlx::migrate!("./migrations").run(&pool).await?;
  
  let app = Router::new()
    .route("/", axum::routing::get(routes::root_path))
    .route("/health", axum::routing::get(routes::health))
    .route("/api/time/all", axum::routing::get(routes::list_submissions))
    .route("/api/time/submit", axum::routing::post(routes::submit_time))
    .route(
      "/api/time/vote/{id}",
      axum::routing::post(routes::vote),
    )
    .route(
      "/api/time/{studio_id}/submissions",
      axum::routing::get(routes::list_submissions_by_studio),
    )
    .route(
      "/api/time/{studio_id}",
      axum::routing::get(routes::get_aggregate),
    )
    .route("/api/user/name", axum::routing::post(routes::set_name))
    .with_state(AppState { pool })
    .layer(CorsLayer::new().allow_origin(Any));
  
  let addr = SocketAddr::from(([0, 0, 0, 0], port()));
  println!("Listening on http://{}", addr);
  let listener = tokio::net::TcpListener::bind(addr).await?;
  axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await?;
  
  Ok(())
}
