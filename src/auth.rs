use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::Json,
};
use serde_json::json;
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub struct BearerUserId(pub Uuid);

impl BearerUserId {
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl<S> FromRequestParts<S> for BearerUserId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Some(auth) = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
        else {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "missing Authorization header" })),
            ));
        };
        const PREFIX: &str = "Bearer ";
        if !auth.starts_with(PREFIX) {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "expected Authorization: Bearer <uuid>" })),
            ));
        }
        let token = auth[PREFIX.len()..].trim();
        let uid = Uuid::parse_str(token).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "invalid bearer token (expected uuid)" })),
            )
        })?;
        Ok(BearerUserId(uid))
    }
}
