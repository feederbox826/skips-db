//! Data access for skip submissions, votes, and per-studio aggregates.

use sqlx::{SqlitePool, Transaction};
use uuid::Uuid;

use crate::models::{AggregateStudio, PublicSubmission};

pub async fn ensure_user(pool: &SqlitePool, user_id: &str) -> sqlx::Result<()> {
  sqlx::query("INSERT OR IGNORE INTO users (user_id) VALUES (?)")
    .bind(user_id)
    .execute(pool)
    .await?;
  Ok(())
}

pub async fn set_user_name(pool: &SqlitePool, user_id: &str, name: &str) -> sqlx::Result<()> {
  ensure_user(pool, user_id).await?;
  sqlx::query("UPDATE users SET name = ? WHERE user_id = ?")
    .bind(name)
    .bind(user_id)
    .execute(pool)
    .await?;
  Ok(())
}

// insert new submission
pub async fn submit_time(
  pool: &SqlitePool,
  user_id: &str,
  studio_id: Uuid,
  skip_seconds: f64,
) -> sqlx::Result<()> {
  // check for existing submissions
  let existing: Option<i64> = sqlx::query_scalar::<_, i64>(
    "SELECT id FROM submissions WHERE studio_id = ? AND skip_seconds = ?",
  )
    .bind(studio_id)
    .bind(skip_seconds)
    .fetch_optional(pool)
    .await?;
  // if exists, upvote instead
  if let Some(id) = existing {
    cast_vote(pool, user_id, id, 1).await?;
    return Ok(());
  }

  let mut tx = pool.begin().await?;
  
  sqlx::query(
    "INSERT INTO submissions (user_id, studio_id, skip_seconds) VALUES (?, ?, ?)
      ON CONFLICT (studio_id, user_id) DO UPDATE SET
        skip_seconds = excluded.skip_seconds,
        created_at = current_date",
  )
    .bind(user_id)
    .bind(studio_id)
    .bind(skip_seconds)
    .execute(&mut *tx)
    .await?;
  
  recompute_studio_aggregate_tx(&mut tx, studio_id).await?;
  
  tx.commit().await?;
  Ok(())
}

pub async fn cast_vote(
  pool: &SqlitePool,
  voter_user_id: &str,
  submission_id: i64,
  value: i64,
) -> sqlx::Result<()> {
  // validate studio exists and user is not self-voting
  let row: Option<(Uuid, String)> =
  sqlx::query_as("SELECT studio_id, user_id FROM submissions WHERE id = ?")
    .bind(submission_id)
    .fetch_optional(pool)
    .await?;
    
  let Some((studio_id, user_id)) = row else {
    return Err(sqlx::Error::RowNotFound);
  };
  // disallow self-votes
  if user_id == voter_user_id {
    return Ok(());
  }

  ensure_user(pool, voter_user_id).await?;
  
  let mut tx = pool.begin().await?;
  sqlx::query(
    "INSERT INTO votes (submission_id, user_id, vote) VALUES (?, ?, ?)
      ON CONFLICT (submission_id, user_id) DO UPDATE SET vote = excluded.vote",
  )
    .bind(submission_id)
    .bind(voter_user_id)
    .bind(value)
    .execute(&mut *tx)
    .await?;
  
  recompute_studio_aggregate_tx(&mut tx, studio_id).await?;
  tx.commit().await?;
  Ok(())
}

pub async fn get_aggregate(
  pool: &SqlitePool,
  studio_id: Uuid,
) -> sqlx::Result<Option<AggregateStudio>> {
  sqlx::query_as::<_, AggregateStudio>(
    "SELECT studio_id, skip_seconds FROM studio_aggregates WHERE studio_id = ?",
  )
    .bind(studio_id)
    .fetch_optional(pool)
    .await
}

pub async fn list_submissions(pool: &SqlitePool) -> sqlx::Result<Vec<PublicSubmission>> {
  let rows = sqlx::query_as::<_, PublicSubmission>(
    r#"SELECT
        s.id,
        s.studio_id,
        s.skip_seconds,
        COALESCE(NULLIF(TRIM(u.name), ''), 'anonymous') AS name,
        COALESCE(SUM(v.vote), 0) AS net_votes
      FROM submissions s
      JOIN users u ON u.user_id = s.user_id
      LEFT JOIN votes v ON v.submission_id = s.id
      GROUP BY s.id, s.studio_id, s.skip_seconds, u.name
      ORDER BY s.id"#,
  )
    .fetch_all(pool)
    .await?;
  Ok(rows)
}


pub async fn list_submissions_for_studio(
  pool: &SqlitePool,
  studio_id: Uuid,
) -> sqlx::Result<Vec<PublicSubmission>> {
  let rows = sqlx::query_as::<_, PublicSubmission>(
    r#"SELECT
        s.id,
        s.studio_id,
        s.skip_seconds,
        COALESCE(NULLIF(TRIM(u.name), ''), 'anonymous') AS name,
        COALESCE(SUM(v.vote), 0) AS net_votes
      FROM submissions s
      JOIN users u ON u.user_id = s.user_id
      LEFT JOIN votes v ON v.submission_id = s.id
      WHERE s.studio_id = ?
      GROUP BY s.id, s.studio_id, s.skip_seconds, u.name
      ORDER BY s.id"#,
  )
    .bind(studio_id)
    .fetch_all(pool)
    .await?;
  Ok(rows)
}

async fn recompute_studio_aggregate_tx(
  tx: &mut Transaction<'_, sqlx::Sqlite>,
  studio_id: Uuid,
) -> sqlx::Result<()> {
  sqlx::query(
    r#"INSERT INTO studio_aggregates (studio_id, skip_seconds)
      WITH scored AS (
        SELECT
          s.id,
          s.studio_id,
          s.skip_seconds,
          COALESCE(SUM(v.vote), 0) AS net_votes
        FROM submissions s
        LEFT JOIN votes v ON v.submission_id = s.id
        WHERE s.studio_id = ?
        GROUP BY s.id, s.studio_id, s.skip_seconds
      ),
      winner AS (
        SELECT studio_id, skip_seconds
        FROM scored
        ORDER BY
          net_votes DESC,
          skip_seconds ASC,
          id DESC
        LIMIT 1
      )
      SELECT studio_id, skip_seconds
      FROM winner
      WHERE 1
      ON CONFLICT (studio_id) DO UPDATE SET
        skip_seconds = excluded.skip_seconds"#,
  )
    .bind(studio_id)
    .execute(&mut **tx)
    .await?;

  sqlx::query(
    r#"DELETE FROM studio_aggregates
      WHERE studio_id = ?
        AND NOT EXISTS (
          SELECT 1 FROM submissions WHERE studio_id = ?
        )"#,
  )
    .bind(studio_id)
    .bind(studio_id)
    .execute(&mut **tx)
    .await?;
  
  Ok(())
}
