use super::model;
use crate::data::{DataError, DbPool};
use crate::ShortCode;
use sqlx::Row;
use crate::data::model::GetClip;
use crate::web::api::ApiKey;

type Result<T> = std::result::Result<T, DataError>;

pub async fn get_clip<M: Into<GetClip>>(model: M, pool: &DbPool) -> Result<model::Clip> {
    let model = model.into();
    let shortcode = model.shortcode.as_str();
    Ok(sqlx::query_as!(
        model::Clip,
        "SELECT * FROM clips WHERE shortcode = ?",
        shortcode
      )
        .fetch_one(pool)
        .await?)
}

// using a model as param to avoid having to pass a whole bunch of clip properties as params
pub async fn new_clip<M: Into<model::NewClip>>(model: M, pool: &DbPool) -> Result<model::Clip> {
    let model = model.into();
    let _ = sqlx::query!(
        r#"INSERT INTO clips (
            clip_id,
            shortcode,
            content,
            title,
            posted,
            expires,
            password,
            hits)
        VALUES(?, ?, ?, ?, ?, ?, ?, ?)"#,
        model.clip_id,
        model.shortcode,
        model.content,
        model.title,
        model.posted,
        model.expires,
        model.password,
        0)
        .execute(pool)
        .await?;

    get_clip(model.shortcode, pool).await
}

pub async fn update_clip<M: Into<model::UpdateClip>>(model: M, pool: &DbPool) -> Result<model::Clip> {
    let model = model.into();
    let _ = sqlx::query!(
        r#"UPDATE clips SET
                content = ?,
                expires = ?,
                password = ?,
                title = ?
           WHERE shortcode = ?"#,
        model.content,
        model.expires,
        model.password,
        model.content,
        model.shortcode
        )
        .execute(pool)
        .await?;

    get_clip(model.shortcode, pool).await
}

pub async fn increase_hit_count(shortcode: &ShortCode, hits: u32, pool: &DbPool) -> Result<()> {
    let shortcode = shortcode.as_str();
    Ok(sqlx::query!(
         "UPDATE clips SET hits = hits + ? WHERE shortcode = ?",
         hits,
         shortcode
       )
        .execute(pool)
        .await
        .map(|_| ())?
    )
}

/// save API_KEY to DB
pub async fn save_api_key(api_key: ApiKey, pool: &DbPool) -> Result<ApiKey> {
    let bytes = api_key.clone().into_inner();
    let _ = sqlx::query!("INSERT INTO api_keys (api_key) VALUES (?)", bytes)
        .execute(pool)
        .await
        .map(|_| ())?;
    Ok(api_key)
}

pub enum RevocationStatus {
    Revoked,
    NotFound,
}

/// remove API_KEY from DB
pub async fn revoke_api_key(api_key: ApiKey, pool: &DbPool) -> Result<RevocationStatus> {
    let bytes = api_key.clone().into_inner();
    Ok(sqlx::query!("DELETE FROM api_keys where api_key == ?", bytes)
        .execute(pool)
        .await
        .map(|res| match res.rows_affected() {
            0 => RevocationStatus::NotFound,
            _ => RevocationStatus::Revoked
        })?
    )
}

pub async fn api_key_is_valid(api_key: ApiKey, pool: &DbPool) -> Result<bool> {
    let bytes = api_key.clone().into_inner();
    Ok(sqlx::query("SELECT FROM api_keys  where api_key == ?")
        .bind(bytes)
        .fetch_one(pool)
        .await
        .map(|row| {
            let count: u32 = row.get(0);
            count > 0
        })?
    )
}
