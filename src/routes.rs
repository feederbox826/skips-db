use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::auth::BearerUserId;
use crate::db;
use crate::models::{AggregateStudio, SetNameBody, SubmitBody, PublicSubmission, VoteBody};
use sqlx::SqlitePool;

#[derive(Clone)]
#[must_use]
pub struct AppState {
  pub pool: SqlitePool,
}

const MAX_NAME_LEN: usize = 256;

pub async fn health() -> &'static str {
  "ok"
}

pub async fn get_aggregate(
  State(state): State<AppState>,
  Path(studio_id_raw): Path<String>,
) -> Result<Json<AggregateStudio>, (StatusCode, Json<serde_json::Value>)> {
  let studio_id = parse_studio_uuid(&studio_id_raw)?;
  let row = db::get_aggregate(&state.pool, studio_id)
    .await
    .map_err(|e| db_err(e))?;
  let Some(aggregate) = row else {
    return Err((
      StatusCode::NOT_FOUND,
      Json(json!({ "error": "no aggregate for studio" })),
    ));
  };
  Ok(Json(aggregate))
}

pub async fn list_submissions(
  State(state): State<AppState>
) -> Result<Json<Vec<PublicSubmission>>, (StatusCode, Json<serde_json::Value>)> {
  let submissions = db::list_submissions(&state.pool)
    .await
    .map_err(|e| db_err(e))?;
  Ok(Json(submissions))
}

pub async fn list_submissions_by_studio(
  State(state): State<AppState>,
  Path(studio_id_raw): Path<String>,
) -> Result<Json<Vec<PublicSubmission>>, (StatusCode, Json<serde_json::Value>)> {
  let studio_id = parse_studio_uuid(&studio_id_raw)?;
  let submissions = db::list_submissions_for_studio(&state.pool, studio_id)
    .await
    .map_err(|e: sqlx::Error| db_err(e))?;
  Ok(Json(submissions))
}

pub async fn submit_time(
  State(state): State<AppState>,
  auth: BearerUserId,
  Json(body): Json<SubmitBody>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
  if !body.skip_seconds.is_finite()
    || body.skip_seconds < 0.0
    || body.skip_seconds >= 60.0
  {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(json!({
        "error": "skip_seconds must be between 0 and 60"
      })),
    ));
  }
  
  let uid = auth.as_str();
  db::ensure_user(&state.pool, &uid)
  .await
  .map_err(|e| db_err(e))?;
  db::submit_time(
    &state.pool,
    &uid,
    body.studio_id,
    body.skip_seconds,
  )
    .await
    .map_err(|e| db_err(e))?;
  Ok(StatusCode::CREATED)
}

pub async fn vote(
  State(state): State<AppState>,
  auth: BearerUserId,
  Path(id): Path<i64>,
  Json(body): Json<VoteBody>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
  if body.value != 1 && body.value != -1 {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(json!({ "error": "value must be 1 or -1" })),
    ));
  }
  let uid = auth.as_str();
  match db::cast_vote(&state.pool, &uid, id, body.value).await {
    Ok(()) => Ok(StatusCode::NO_CONTENT),
    Err(sqlx::Error::RowNotFound) => Err((
      StatusCode::NOT_FOUND,
      Json(json!({ "error": "submission not found" })),
    )),
    Err(e) => Err(db_err(e)),
  }
}

pub async fn set_name(
  State(state): State<AppState>,
  auth: BearerUserId,
  Json(body): Json<SetNameBody>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
  let name = body.name.trim();
  if name.len() > MAX_NAME_LEN {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(json!(
        { "error": format!("name can be at most {} characters", MAX_NAME_LEN) }
      )),
    ));
  }
  let uid = auth.as_str();
  db::set_user_name(&state.pool, &uid, name)
  .await
  .map_err(|e| db_err(e))?;
  Ok(StatusCode::NO_CONTENT)
}

fn db_err(e: sqlx::Error) -> (StatusCode, Json<serde_json::Value>) {
  eprintln!("database error: {e}");
  (
    StatusCode::INTERNAL_SERVER_ERROR,
    Json(json!({ "error": "internal server error" })),
  )
}

fn parse_studio_uuid(raw: &str) -> Result<Uuid, (StatusCode, Json<serde_json::Value>)> {
  Uuid::parse_str(raw.trim()).map_err(|_| {
    (
      StatusCode::BAD_REQUEST,
      Json(json!({ "error": "studio_id must be a valid UUID" })),
    )
  })
}
