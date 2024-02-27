use anyhow::Result;
use axum::{
    http::HeaderValue,
    response::{Html, IntoResponse},
    routing::post,
    Router,
};
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};

use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};

mod adifgen;
mod strconv;

use adifgen::{ADIFData, ADIFRecord};

#[tokio::main]
async fn main() {
    let origins = ["https:://sotalive.net".parse::<HeaderValue>().unwrap()];
    let serve_dir = ServeDir::new("static").not_found_service(ServeFile::new("static/index.html"));
    let app = Router::new()
        .route("/api/ADIFcheck", post(adifcheck_handler))
        .route("/api/ADIFgen", post(adifgen_handler))
        .nest_service("/", serve_dir.clone())
        .layer(CorsLayer::new().allow_origin(origins));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

#[derive(TryFromMultipart)]
struct ADIFGenRequest {
    activator_call: String,
    operator: String,
    my_qth: String,
    references: String,
    his_qth: String,
    #[form_data(limit = "20MiB")]
    filename: axum::body::Bytes,
}

async fn adifcheck_handler(
    TypedMultipart(ADIFGenRequest {
        activator_call,
        operator,
        my_qth,
        references,
        his_qth,
        filename,
    }): TypedMultipart<ADIFGenRequest>,
) -> impl IntoResponse {
    let (log, _, _) = encoding_rs::SHIFT_JIS.decode(&filename);
    let res = adifgen::adifcheck(
        &activator_call,
        &operator,
        &my_qth,
        &references,
        &his_qth,
        &log,
    );
    serde_json::to_string(&res).unwrap()
}

async fn adifgen_handler(
    TypedMultipart(ADIFGenRequest {
        activator_call,
        operator,
        my_qth,
        references,
        his_qth,
        filename,
    }): TypedMultipart<ADIFGenRequest>,
) -> impl IntoResponse {
    println!(
        "Generator log for {} on refs {}",
        activator_call, references
    );
    Html("<h1>Hello, world!</h1>")
}
