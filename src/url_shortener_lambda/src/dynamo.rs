use crate::error::AppError;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{config::Region, types::AttributeValue, Client};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_dynamo::{from_item, to_item};
use std::env;
use std::fmt::Debug;
use tokio::sync::OnceCell;
use tracing::instrument;

static DYNAMO_CLIENT: OnceCell<Client> = OnceCell::const_new();
static TABLE_NAME: Lazy<String> =
    Lazy::new(|| env::var("table_name").expect("Missing table_name environment variable"));

async fn build_dynamo_client() -> Client {
    let region_provider = RegionProviderChain::default_provider().or_else(Region::new("us-west-2"));
    let sdk_config = aws_config::from_env().region(region_provider).load().await;
    let ddb_config = aws_sdk_dynamodb::Config::new(&sdk_config);

    Client::from_conf(ddb_config)
}

#[instrument]
pub async fn get_dynamo_client() -> Client {
    DYNAMO_CLIENT
        .get_or_init(|| async { build_dynamo_client().await })
        .await
        .to_owned()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UrlItem {
    pub slug: String,
    pub url: String,
    pub permanent_redirect: bool,
    pub ttl: Option<i64>,
    pub created_at: DateTime<Utc>,
}

pub async fn create_url(dynamo: &Client, url_item: UrlItem) -> Result<(), AppError> {
    let item = to_item(url_item)?;

    dynamo
        .put_item()
        .table_name(TABLE_NAME.as_str())
        .set_item(Some(item))
        .send()
        .await
        .map_err(|_| AppError::AwsSdkError)?;

    Ok(())
}

#[instrument(skip(dynamo))]
pub async fn get_url(dynamo: &Client, slug: &str) -> Result<Option<UrlItem>, AppError> {
    let result = dynamo
        .query()
        .table_name(TABLE_NAME.as_str())
        .key_condition_expression("#DDB_slug = :pkey")
        .expression_attribute_names("#DDB_slug", "slug")
        .expression_attribute_values(":pkey", AttributeValue::S(String::from(slug)))
        .limit(1)
        .send()
        .await
        .map_err(|_| AppError::AwsSdkError)?;

    let query_item = result.items().map(|items| items.get(0)).flatten();

    let Some(item) = query_item else {
        return Ok(None)
    };

    let result: UrlItem = from_item(item.clone())?;

    Ok(Some(result))
}
