use axum::{
    routing::{get, post},
    Extension, Router,
};
use dynamo::get_dynamo_client;
use handlers::{create_shorten, get_shorten, index, redirect_shorten};
use lambda_http::{run, tower::MakeService, Error, Service};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};

mod dynamo;
mod error;
mod handlers;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::ERROR)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .json()
        .init();

    let ddb_client = get_dynamo_client().await;
    let app = Router::new()
        .route("/api/shorten", post(create_shorten))
        .route("/api/shorten/:id", get(get_shorten))
        .route("/:id", get(redirect_shorten))
        .route("/", get(index))
        .layer(Extension(ddb_client.clone()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(tower_http::LatencyUnit::Millis),
                ),
        );

 let app =  Router::new().layer(make_router(ddb_client));

        tower::
    run(app).await
}

fn make_router(ddb_client: aws_sdk_dynamodb::Client) -> Router {
    Router::new()
        .route("/api/shorten", post(create_shorten))
        .route("/api/shorten/:id", get(get_shorten))
        .route("/:id", get(redirect_shorten))
        .route("/", get(index))
        .layer(Extension(ddb_client))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(tower_http::LatencyUnit::Millis),
                ),
        )
}
