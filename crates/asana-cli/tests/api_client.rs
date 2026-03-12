//! Integration coverage for the asynchronous API client.

use asana_cli::api::{ApiClient, ApiError, AuthToken, ListResponse};
use futures_util::StreamExt;
use mockito::{Matcher, Server};
use secrecy::SecretString;
use serde::Deserialize;
use serde_json::Value;
use tempfile::TempDir;
use tokio::time::Duration;

#[derive(Debug, Deserialize)]
struct Workspace {
    gid: String,
    name: String,
}

#[tokio::test]
async fn paginate_workspaces_streams_all_pages() {
    {
        let mut server = Server::new_async().await;
        let _first_page = server
            .mock("GET", "/workspaces")
            .with_status(200)
            .with_body(
                r#"{
                    "data": [
                        { "gid": "123", "name": "Engineering" },
                        { "gid": "456", "name": "Design" }
                    ],
                    "next_page": { "offset": "after-first", "path": "/workspaces" }
                }"#,
            )
            .create();
        let _second_page = server
            .mock("GET", "/workspaces")
            .match_query(Matcher::UrlEncoded("offset".into(), "after-first".into()))
            .with_status(200)
            .with_body(
                r#"{
                    "data": [
                        { "gid": "789", "name": "Operations" }
                    ]
                }"#,
            )
            .create();

        let cache = TempDir::new().expect("temporary cache dir");
        let token = AuthToken::new(SecretString::new("workspace-token".into()));
        let base_url = server.url();
        let client = ApiClient::builder(token)
            .base_url(base_url)
            .cache_dir(cache.path().join("cache"))
            .build()
            .expect("client initialises");

        let stream = client.paginate::<Workspace>("/workspaces", vec![]);
        tokio::pin!(stream);

        let mut workspaces = Vec::new();
        while let Some(page) = stream.next().await {
            let page = page.expect("page result");
            workspaces.extend(
                page.into_iter()
                    .map(|workspace| (workspace.gid, workspace.name)),
            );
        }

        assert_eq!(
            workspaces,
            vec![
                ("123".to_string(), "Engineering".to_string()),
                ("456".to_string(), "Design".to_string()),
                ("789".to_string(), "Operations".to_string()),
            ]
        );
        drop(server);
    }
}

#[tokio::test]
async fn rate_limit_recovers_after_retry() {
    {
        let mut server = Server::new_async().await;
        let _first = server
            .mock("GET", "/users/me")
            .with_status(429)
            .with_header("Retry-After", "0.05")
            .with_body(r#"{ "errors": [ { "message": "Too many requests" } ] }"#)
            .create();
        let _second = server
            .mock("GET", "/users/me")
            .with_status(200)
            .with_header("X-RateLimit-Limit", "150")
            .with_header("X-RateLimit-Remaining", "149")
            .with_body(r#"{ "data": { "name": "Rate Limited User" } }"#)
            .create();

        let cache = TempDir::new().expect("temporary cache dir");
        let token = AuthToken::new(SecretString::new("rate-limit-token".into()));
        let base_url = server.url();
        let client = ApiClient::builder(token)
            .base_url(base_url)
            .cache_dir(cache.path().join("cache"))
            .retry_base_delay(Duration::from_millis(10))
            .max_retries(3)
            .build()
            .expect("client initialises");

        let user = client
            .get_current_user()
            .await
            .expect("rate limited call eventually succeeds");
        assert_eq!(user["data"]["name"], "Rate Limited User");

        let info = client
            .rate_limit_info()
            .await
            .expect("rate limit telemetry captured");
        assert_eq!(info.limit, Some(150));
        assert_eq!(info.remaining, Some(149));
        assert!(info.retry_after.is_none());
        drop(server);
    }
}

#[tokio::test]
async fn rate_limit_failure_surfaces_retry_after() {
    {
        let mut server = Server::new_async().await;
        let _first = server
            .mock("GET", "/users/me")
            .with_status(429)
            .with_header("Retry-After", "0.05")
            .with_body("rate limited")
            .create();
        let _second = server
            .mock("GET", "/users/me")
            .with_status(429)
            .with_header("Retry-After", "0.05")
            .with_body("still rate limited")
            .create();

        let cache = TempDir::new().expect("temporary cache dir");
        let token = AuthToken::new(SecretString::new("rate-limit-failure".into()));
        let base_url = server.url();
        let client = ApiClient::builder(token)
            .base_url(base_url)
            .cache_dir(cache.path().join("cache"))
            .retry_base_delay(Duration::from_millis(10))
            .max_retries(1)
            .build()
            .expect("client initialises");

        let err = client
            .get_current_user()
            .await
            .expect_err("should rate limit");
        match err {
            ApiError::RateLimited { retry_after, body } => {
                assert!(retry_after >= Duration::from_millis(10));
                assert!(body.contains("rate limited"));
            }
            other => panic!("expected rate limited error, got {other:?}"),
        }
        drop(server);
    }
}

#[tokio::test]
async fn optional_live_smoke_test() {
    let token = match std::env::var("ASANA_CLI_TEST_TOKEN") {
        Ok(value) if !value.is_empty() => SecretString::new(value.into()),
        _ => return,
    };

    let base_url = std::env::var("ASANA_BASE_URL")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "https://app.asana.com/api/1.0".to_string());

    let cache = TempDir::new().expect("temporary cache dir");
    let client = ApiClient::builder(AuthToken::new(token))
        .base_url(base_url)
        .cache_dir(cache.path().join("cache"))
        .build()
        .expect("client initialises");

    let response: ListResponse<Value> = client
        .get_json("/workspaces", &[])
        .await
        .expect("live API call returns data");
    assert!(
        !response.data.is_empty(),
        "live workspace listing should return at least one workspace"
    );
}
