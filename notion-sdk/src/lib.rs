//! [![github]](https://github.com/emo-crab/notion-rs)&ensp;[![crates-io]](https://crates.io/crates/notion-sdk)&ensp;[![docs-rs]](crate)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//!
//!
//! UnOfficial Notion SDK mplemented by rust
//!
//!
//! ## Examples
//! ```rust,no_run
//! use notion_sdk::NotionApi;
//! async fn main(){
//!     let notion = NotionApi::new("token")?;
//!     let me = notion.users_me().await;
//!     println!("{:#?}", me);
//! }
//!
//! ```
pub mod block;
pub mod comment;
pub mod common;
pub mod database;
pub mod error;
pub mod pages;
pub mod pagination;
pub mod search;
pub mod user;

use crate::error::Error;
use crate::pagination::Object;
use reqwest::{ClientBuilder, RequestBuilder};

use log::{info,debug};

const NOTION_API_VERSION: &str = "2022-02-22";

/// Notion Api Client
#[derive(Debug, Clone)]
pub struct NotionApi {
    base_path: String,
    client: reqwest::Client,
}

/// new a notion api client with api token
impl NotionApi {
    pub fn new<T>(api_token: T) -> Result<Self, Error> where
        T: Into<String> + std::fmt::Display, {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Notion-Version",
            reqwest::header::HeaderValue::from_static(NOTION_API_VERSION),
        );
        let mut auth_value = reqwest::header::HeaderValue::from_str(&format!("Bearer {api_token}"))
            .map_err(|source| Error::InvalidApiToken { source })?;
        auth_value.set_sensitive(true);
        headers.insert(reqwest::header::AUTHORIZATION, auth_value);
        let api_client = ClientBuilder::new()
            .default_headers(headers)
            .build()
            .map_err(|source| Error::ErrorBuildingClient { source })?;
        Ok(NotionApi {
            base_path: "https://api.notion.com/v1".to_owned(),
            client: api_client,
        })
    }
}

impl NotionApi {
    async fn request(&self, request: RequestBuilder) -> Result<Object, Error> {
        // Build the actual HTTP request from the builder
        let request = request.build()?;

        // Print request method and URL
        debug!("🔸 Request Method: {}", request.method());
        info!("🔸 Request URL: {}", request.url());

        // Print all request headers
        info!("🔸 Request Headers:");
        for (key, value) in request.headers().iter() {
            info!("    {}: {:?}", key, value);
        }

        // Attempt to print the request body (if present and accessible)
        if let Some(body) = request.body() {
            if let Some(bytes) = body.as_bytes() {
                match std::str::from_utf8(bytes) {
                    Ok(text) => println!("🔸 Request Body:\n{}", text),
                    Err(_) => info!("🔸 Request body is not valid UTF-8 and cannot be displayed as text"),
                }
            } else {
                info!("🔸 Request body is not accessible as raw bytes (possibly streamed)");
            }
        } else {
            info!("🔸 No request body");
        }

        // Execute the HTTP request
        let response = self
            .client
            .execute(request)
            .await
            .map_err(|source| Error::RequestFailed { source })?;

        // Read the full response body as a string
        let json = response
            .text()
            .await
            .map_err(|source| Error::ResponseIoError { source })?;

        // Optionally print the raw response body
        info!("🔹 Response Body:\n{}", json);

        // Parse the JSON response into Object
        let result =
            serde_json::from_str(&json).map_err(|source| Error::JsonParseError { source })?;

        // Handle API errors or return the result
        match result {
            Object::Error { error } => Err(Error::ApiError { error }),
            response => Ok(response),
        }
    }
}
