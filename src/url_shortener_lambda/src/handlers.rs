use crate::{
    dynamo::{create_url, get_url, UrlItem},
    error::AppError,
};
use aws_sdk_dynamodb::Client;
use axum::{
    extract::Path,
    response::{IntoResponse, Redirect},
    Extension, Json,
};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use tracing::instrument;
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
pub struct ShortenURLInput {
    #[validate(url, length(max = 256))]
    pub url: String,
    #[validate(custom = "validate_expiration")]
    expiration_date: Option<DateTime<Utc>>,
    permanent_redirect: Option<bool>,
}

fn validate_expiration(item: &DateTime<Utc>) -> Result<(), ValidationError> {
    let now = Utc::now();
    let min_allowed = now + Duration::minutes(5);
    let max_allowed = now + Duration::days(365);

    if *item > min_allowed && *item < max_allowed {
        return Ok(());
    }

    let mut err = ValidationError::new("Invalid expiration_date");
    err.message = Some(Cow::Borrowed(
        "Must be at least 5 minutes from in the future, and not more than a year",
    ));
    err.add_param(Cow::Borrowed("value"), item);
    err.add_param(Cow::Borrowed("min_allowed"), &min_allowed);
    err.add_param(Cow::Borrowed("max_allowed"), &max_allowed);

    return Err(err);
}

impl Into<UrlItem> for ShortenURLInput {
    fn into(self) -> UrlItem {
        UrlItem {
            slug: nanoid!(10),
            url: self.url,
            permanent_redirect: self.permanent_redirect.unwrap_or(false),
            ttl: self.expiration_date.map(|i| i.timestamp()),
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ShortenedUrlOutput {
    pub slug: String,
    pub url: String,
    pub expiration_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub permanent_redirect: bool,
}

impl From<UrlItem> for ShortenedUrlOutput {
    fn from(value: UrlItem) -> Self {
        Self {
            slug: value.slug,
            url: value.url,
            expiration_date: value
                .ttl
                .map(|i| NaiveDateTime::from_timestamp_opt(i, 0))
                .flatten()
                .map(|i| DateTime::from_utc(i, Utc)),
            created_at: value.created_at,
            permanent_redirect: value.permanent_redirect,
        }
    }
}

#[instrument(skip(dynamo))]
pub async fn create_shorten(
    dynamo: Extension<Client>,
    Json(input): Json<ShortenURLInput>,
) -> Result<Json<ShortenedUrlOutput>, AppError> {
    input.validate()?;

    let item: UrlItem = input.into();

    create_url(&dynamo, item.clone()).await?;

    let output: ShortenedUrlOutput = item.into();

    Ok(Json(output))
}

#[instrument(skip(dynamo))]
pub async fn get_shorten(
    dynamo: Extension<Client>,
    slug: Path<String>,
) -> Result<Json<ShortenedUrlOutput>, AppError> {
    let item = get_url(&dynamo, &slug).await?.ok_or(AppError::NotFound)?;

    Ok(Json(item.into()))
}

#[instrument(skip(dynamo))]
pub async fn redirect_shorten(
    dynamo: Extension<Client>,
    slug: Path<String>,
) -> Result<Redirect, AppError> {
    let Some (item) = get_url(&dynamo, &slug).await? else {
        return Ok(Redirect::to("/"))
    };

    match item.permanent_redirect {
        true => Ok(Redirect::permanent(&item.url)),
        false => Ok(Redirect::to(&item.url)),
    }
}

pub async fn index() -> impl IntoResponse {
    format!("Hello World!")
}
