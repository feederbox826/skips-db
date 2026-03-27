use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AggregateJson {
  pub studio_id: Uuid,
  pub skip_seconds: Option<f64>,
  pub no_intro: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SubmissionPublic {
  pub id: i64,
  pub studio_id: Uuid,
  pub skip_seconds: Option<f64>,
  pub no_intro: Option<bool>,
  pub name: String,
  pub net_votes: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct SubmitBody {
  pub studio_id: Uuid,
  pub skip_seconds: Option<f64>,
  pub no_intro: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
pub struct VoteBody {
  pub value: i64,
}

#[derive(Debug, serde::Deserialize)]
pub struct SetNameBody {
  pub name: String,
}