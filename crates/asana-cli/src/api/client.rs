//! Core asynchronous HTTP client for interacting with Asana's REST API.

use crate::api::{
    auth::AuthToken,
    error::{ApiError, RateLimitInfo},
    pagination::ListResponse,
};
use async_stream::try_stream;
use base64::{Engine as _, engine::general_purpose};
use directories::ProjectDirs;
use futures_core::Stream;
use reqwest::{
    Method, StatusCode,
    header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, RETRY_AFTER, USER_AGENT},
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{fs, sync::RwLock, time::sleep};
use tracing::{debug, warn};

const VERSION: &str = match option_env!("CARGO_PKG_VERSION") {
    Some(version) => version,
    None => "unknown",
};

/// In-memory cache entry.
#[derive(Clone)]
struct CacheEntry {
    expires_at: Instant,
    value: Arc<Vec<u8>>,
}

/// On-disk cache entry representation.
#[derive(Debug, Serialize, Deserialize)]
struct DiskCacheEntry {
    expires_at: u64,
    body: String,
}

/// Configurable options for the API client.
#[derive(Debug, Clone)]
pub struct ApiClientOptions {
    /// Base URL for the Asana API.
    pub base_url: String,
    /// User agent string sent with every request.
    pub user_agent: String,
    /// Total request timeout applied to HTTP calls.
    pub timeout: Duration,
    /// Maximum number of retry attempts for transient failures.
    pub max_retries: usize,
    /// Initial backoff delay applied between retries.
    pub retry_base_delay: Duration,
    /// Time-to-live for cached responses.
    pub cache_ttl: Duration,
    /// Directory used to persist cached responses across runs.
    pub cache_dir: PathBuf,
    /// Whether the client should avoid network calls and use cached data only.
    pub offline: bool,
}

impl ApiClientOptions {
    /// Construct options with a specific base URL.
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            ..Self::default()
        }
    }

    /// Override the cache directory used for disk persistence.
    #[must_use]
    pub fn with_cache_dir(mut self, cache_dir: PathBuf) -> Self {
        self.cache_dir = cache_dir;
        self
    }

    /// Override the request timeout.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Override retry attempts.
    #[must_use]
    pub const fn with_max_retries(mut self, retries: usize) -> Self {
        self.max_retries = retries;
        self
    }

    /// Override retry backoff base delay.
    #[must_use]
    pub const fn with_retry_base_delay(mut self, delay: Duration) -> Self {
        self.retry_base_delay = delay;
        self
    }

    /// Override cache TTL.
    #[must_use]
    pub const fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Start the client in offline mode.
    #[must_use]
    pub const fn with_offline(mut self, offline: bool) -> Self {
        self.offline = offline;
        self
    }
}

impl Default for ApiClientOptions {
    fn default() -> Self {
        let base_url = crate::config::DEFAULT_API_BASE_URL.to_string();
        let user_agent = format!("asana-cli/{VERSION}");
        let cache_dir = default_cache_dir();
        Self {
            base_url,
            user_agent,
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_base_delay: Duration::from_millis(500),
            cache_ttl: Duration::from_secs(300),
            cache_dir,
            offline: false,
        }
    }
}

fn default_cache_dir() -> PathBuf {
    ProjectDirs::from("com", "asana", "asana-cli").map_or_else(
        || {
            let mut path = std::env::temp_dir();
            path.push("asana-cli-cache");
            path
        },
        |dirs| dirs.data_local_dir().join("cache"),
    )
}

/// Builder for [`ApiClient`].
pub struct ApiClientBuilder {
    token: AuthToken,
    options: ApiClientOptions,
}

impl ApiClientBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new(token: AuthToken) -> Self {
        Self {
            token,
            options: ApiClientOptions::default(),
        }
    }

    /// Set the base URL.
    #[must_use]
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.options.base_url = base_url.into();
        self
    }

    /// Override the user agent.
    #[must_use]
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.options.user_agent = user_agent.into();
        self
    }

    /// Override the cache directory.
    #[must_use]
    pub fn cache_dir(mut self, cache_dir: PathBuf) -> Self {
        self.options.cache_dir = cache_dir;
        self
    }

    /// Override timeout.
    #[must_use]
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.options.timeout = timeout;
        self
    }

    /// Override retry attempts.
    #[must_use]
    pub const fn max_retries(mut self, retries: usize) -> Self {
        self.options.max_retries = retries;
        self
    }

    /// Override retry base delay.
    #[must_use]
    pub const fn retry_base_delay(mut self, delay: Duration) -> Self {
        self.options.retry_base_delay = delay;
        self
    }

    /// Override cache TTL.
    #[must_use]
    pub const fn cache_ttl(mut self, ttl: Duration) -> Self {
        self.options.cache_ttl = ttl;
        self
    }

    /// Configure offline mode.
    #[must_use]
    pub const fn offline(mut self, offline: bool) -> Self {
        self.options.offline = offline;
        self
    }

    /// Finalise the builder, creating an [`ApiClient`].
    ///
    /// # Errors
    ///
    /// Returns an error if the cache directory cannot be created or if the HTTP client fails to initialize.
    pub fn build(self) -> Result<ApiClient, ApiError> {
        ApiClient::with_options(self.token, self.options)
    }
}

/// Asynchronous Asana API client handling retries, rate limiting, and caching.
pub struct ApiClient {
    http: reqwest::Client,
    token: AuthToken,
    options: ApiClientOptions,
    memory_cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    offline: AtomicBool,
    rate_limit: Arc<RwLock<Option<RateLimitInfo>>>,
}

impl Clone for ApiClient {
    fn clone(&self) -> Self {
        Self {
            http: self.http.clone(),
            token: self.token.clone(),
            options: self.options.clone(),
            memory_cache: Arc::clone(&self.memory_cache),
            offline: AtomicBool::new(self.offline.load(Ordering::Relaxed)),
            rate_limit: Arc::clone(&self.rate_limit),
        }
    }
}

impl ApiClient {
    /// Create a builder for configuring the client.
    #[must_use]
    pub fn builder(token: AuthToken) -> ApiClientBuilder {
        ApiClientBuilder::new(token)
    }

    /// Construct a client with default options.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache directory cannot be created or if the HTTP client fails to initialize.
    pub fn new(token: AuthToken) -> Result<Self, ApiError> {
        Self::with_options(token, ApiClientOptions::default())
    }

    /// Construct a client with specific options.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache directory cannot be created or if the HTTP client fails to initialize.
    pub fn with_options(token: AuthToken, options: ApiClientOptions) -> Result<Self, ApiError> {
        std::fs::create_dir_all(&options.cache_dir)?;

        let mut default_headers = HeaderMap::new();
        default_headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        let user_agent_value = HeaderValue::from_str(&options.user_agent)
            .unwrap_or_else(|_| HeaderValue::from_static("asana-cli"));
        default_headers.insert(USER_AGENT, user_agent_value);

        let http = reqwest::Client::builder()
            .timeout(options.timeout)
            .connect_timeout(Duration::from_secs(10))
            .default_headers(default_headers)
            .build()?;

        let offline = options.offline;
        Ok(Self {
            http,
            token,
            options,
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
            offline: AtomicBool::new(offline),
            rate_limit: Arc::new(RwLock::new(None)),
        })
    }

    /// Update offline mode at runtime.
    pub fn set_offline(&self, offline: bool) {
        self.offline.store(offline, Ordering::Relaxed);
    }

    /// Determine if offline mode is active.
    #[must_use]
    pub fn is_offline(&self) -> bool {
        self.offline.load(Ordering::Relaxed)
    }

    /// Retrieve the most recent rate-limit information captured from the API.
    #[must_use]
    pub async fn rate_limit_info(&self) -> Option<RateLimitInfo> {
        let guard = self.rate_limit.read().await;
        guard.clone()
    }

    /// Return the base URL currently configured.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.options.base_url
    }

    /// Retrieve JSON from an endpoint and deserialize into `T`.
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails, the response is invalid, or deserialization fails.
    pub async fn get_json<T>(&self, path: &str, query: &[(&str, &str)]) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let query_pairs = build_query_pairs(query);
        self.get_json_with_pairs(path, query_pairs).await
    }

    /// Retrieve the current authenticated user (`/users/me`).
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails, authentication is invalid, or the response cannot be parsed.
    pub async fn get_current_user(&self) -> Result<Value, ApiError> {
        self.get_json("/users/me", &[]).await
    }

    /// POST helper for JSON endpoints returning a structured payload.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails, the network request fails, or the response cannot be deserialized.
    pub async fn post_json<T, R>(&self, path: &str, body: &T) -> Result<R, ApiError>
    where
        T: Serialize + ?Sized + Sync,
        R: DeserializeOwned,
    {
        self.post_json_with_query(path, Vec::new(), body).await
    }

    /// POST helper accepting explicit query parameters.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails, the network request fails, or the response cannot be deserialized.
    pub async fn post_json_with_query<T, R>(
        &self,
        path: &str,
        query_pairs: Vec<(String, String)>,
        body: &T,
    ) -> Result<R, ApiError>
    where
        T: Serialize + ?Sized + Sync,
        R: DeserializeOwned,
    {
        let bytes = self
            .execute_serialized(Method::POST, path, query_pairs, Some(body))
            .await?;
        Self::parse_response(path, &bytes)
    }

    /// POST helper for endpoints that do not return a payload.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails or the network request fails.
    pub async fn post_void<T>(&self, path: &str, body: &T) -> Result<(), ApiError>
    where
        T: Serialize + ?Sized + Sync,
    {
        let _ = self
            .execute_serialized(Method::POST, path, Vec::new(), Some(body))
            .await?;
        Ok(())
    }

    /// PUT helper for JSON endpoints returning a structured payload.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails, the network request fails, or the response cannot be deserialized.
    pub async fn put_json<T, R>(&self, path: &str, body: &T) -> Result<R, ApiError>
    where
        T: Serialize + ?Sized + Sync,
        R: DeserializeOwned,
    {
        self.put_json_with_query(path, Vec::new(), body).await
    }

    /// PUT helper accepting explicit query parameters.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails, the network request fails, or the response cannot be deserialized.
    pub async fn put_json_with_query<T, R>(
        &self,
        path: &str,
        query_pairs: Vec<(String, String)>,
        body: &T,
    ) -> Result<R, ApiError>
    where
        T: Serialize + ?Sized + Sync,
        R: DeserializeOwned,
    {
        let bytes = self
            .execute_serialized(Method::PUT, path, query_pairs, Some(body))
            .await?;
        Self::parse_response(path, &bytes)
    }

    /// DELETE helper ignoring the response payload.
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails.
    pub async fn delete(
        &self,
        path: &str,
        query_pairs: Vec<(String, String)>,
    ) -> Result<(), ApiError> {
        let _ = self
            .execute(Method::DELETE, path, query_pairs, None)
            .await?;
        Ok(())
    }

    /// POST multipart form data.
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails or the response cannot be deserialized.
    pub async fn post_multipart<R>(
        &self,
        path: &str,
        form: reqwest::multipart::Form,
    ) -> Result<R, ApiError>
    where
        R: DeserializeOwned,
    {
        let url = format!("{}{path}", self.options.base_url);

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token.expose()))
            .multipart(form)
            .send()
            .await?;

        let status = response.status();
        let bytes = response.bytes().await?;

        if !status.is_success() {
            let message = String::from_utf8_lossy(&bytes).to_string();
            return Err(ApiError::Http {
                status,
                message,
                details: None,
            });
        }

        Self::parse_response(path, &bytes)
    }

    /// Download file from URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the network request fails or the download fails.
    pub async fn download_file(&self, url: &str) -> Result<Vec<u8>, ApiError> {
        let response = self
            .http
            .get(url)
            .header("Authorization", format!("Bearer {}", self.token.expose()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Download failed".to_string());
            return Err(ApiError::Http {
                status,
                message,
                details: None,
            });
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// Stream paginated endpoints as a series of pages (`Vec<T>`).
    pub fn paginate<T>(
        &self,
        path: impl Into<String>,
        query: Vec<(String, String)>,
    ) -> impl Stream<Item = Result<Vec<T>, ApiError>> + '_
    where
        T: DeserializeOwned + Send + 'static,
    {
        self.paginate_with_limit(path, query, None)
    }

    /// Stream paginated endpoints with an optional global item limit.
    pub fn paginate_with_limit<T>(
        &self,
        path: impl Into<String>,
        query: Vec<(String, String)>,
        max_items: Option<usize>,
    ) -> impl Stream<Item = Result<Vec<T>, ApiError>> + '_
    where
        T: DeserializeOwned + Send + 'static,
    {
        let path = path.into();
        let client = self.clone();

        try_stream! {
            let mut next_offset: Option<String> = None;
            let mut emitted: usize = 0;
            loop {
                if let Some(max) = max_items
                    && emitted >= max {
                        break;
                    }

                let mut query_pairs = query.clone();
                if let Some(offset) = next_offset.clone() {
                    query_pairs.push(("offset".to_string(), offset));
                }

                let response: ListResponse<T> = match client
                    .get_json_with_pairs(&path, query_pairs.clone())
                    .await
                {
                    Ok(resp) => resp,
                    Err(ApiError::Http { status: StatusCode::BAD_REQUEST, details, message })
                        if is_offset_expired(details.as_ref(), &message) =>
                    {
                        break;
                    }
                    Err(err) => {
                        Err(err)?;
                        unreachable!();
                    }
                };

                let mut items = response.data;
                let next_offset_candidate = response
                    .next_page
                    .as_ref()
                    .and_then(|meta| meta.offset.clone());

                if let Some(max) = max_items
                    && emitted + items.len() > max {
                        items.truncate(max - emitted);
                    }

                emitted += items.len();
                let continue_after_page = next_offset_candidate.is_some()
                    && max_items.is_none_or(|max| emitted < max);

                yield items;

                if !continue_after_page {
                    break;
                }

                next_offset = next_offset_candidate;
            }
        }
    }

    pub(crate) async fn get_json_with_pairs<T>(
        &self,
        path: &str,
        query_pairs: Vec<(String, String)>,
    ) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let bytes = self.execute(Method::GET, path, query_pairs, None).await?;
        Self::parse_response(path, &bytes)
    }

    async fn execute_serialized<T>(
        &self,
        method: Method,
        path: &str,
        query_pairs: Vec<(String, String)>,
        body: Option<&T>,
    ) -> Result<Vec<u8>, ApiError>
    where
        T: Serialize + ?Sized + Sync,
    {
        let json_body = match body {
            Some(payload) => Some(serde_json::to_value(payload)?),
            None => None,
        };
        self.execute(method, path, query_pairs, json_body).await
    }

    fn parse_response<T>(path: &str, bytes: &[u8]) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        if bytes.is_empty() {
            return Err(ApiError::Other(format!("empty response body for {path}")));
        }
        let value: Value = serde_json::from_slice(bytes)?;
        validate_response_schema(&value)?;
        Ok(serde_json::from_value::<T>(value)?)
    }

    #[allow(clippy::too_many_lines)]
    async fn execute(
        &self,
        method: Method,
        path: &str,
        query_pairs: Vec<(String, String)>,
        body: Option<Value>,
    ) -> Result<Vec<u8>, ApiError> {
        let mut cache_key = None;
        if method == Method::GET {
            let key = Self::build_cache_key(&method, path, &query_pairs);
            if let Some(bytes) = self.get_from_cache(&key).await? {
                return Ok(bytes);
            }
            cache_key = Some(key);
            if self.is_offline() {
                return Err(ApiError::Offline {
                    resource: path.to_string(),
                });
            }
        }

        let url = self.build_url(path);
        let mut attempt = 0usize;
        let max_retries = self.options.max_retries;
        let body_clone = body.clone();

        loop {
            let mut request = self.http.request(method.clone(), &url);
            request = request.header(AUTHORIZATION, format!("Bearer {}", self.token.expose()));
            if !query_pairs.is_empty() {
                request = request.query(&query_pairs);
            }
            if let Some(ref json) = body_clone {
                request = request.json(json);
            }

            let response = request.send().await;
            match response {
                Err(err) => {
                    if (err.is_timeout() || err.is_connect()) && attempt < max_retries {
                        let delay = self.backoff_delay(attempt);
                        debug!("retrying after network error: {err}; sleeping {delay:?}");
                        sleep(delay).await;
                        attempt += 1;
                        continue;
                    }
                    return Err(ApiError::Network(err));
                }
                Ok(resp) => {
                    if resp.status().is_success() {
                        let headers = resp.headers().clone();
                        let bytes = resp.bytes().await?.to_vec();
                        if let Some(info) = Self::extract_rate_limit_headers(&headers) {
                            let mut guard = self.rate_limit.write().await;
                            *guard = Some(info);
                        }
                        if let Some(ref key) = cache_key {
                            self.write_cache(key, &bytes).await?;
                        }
                        return Ok(bytes);
                    }

                    let status = resp.status();

                    if status == StatusCode::TOO_MANY_REQUESTS {
                        if let Some(info) = Self::extract_rate_limit_headers(resp.headers()) {
                            let mut guard = self.rate_limit.write().await;
                            *guard = Some(info.clone());
                        }
                        let retry_after = Self::parse_retry_after(resp.headers())
                            .unwrap_or_else(|| self.backoff_delay(attempt));
                        if attempt < max_retries {
                            debug!(
                                "rate limited, waiting {:?} before retry (attempt {})",
                                retry_after,
                                attempt + 1
                            );
                            sleep(retry_after).await;
                            attempt += 1;
                            continue;
                        }
                        let body = resp.text().await.unwrap_or_default();
                        return Err(ApiError::RateLimited { retry_after, body });
                    }

                    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
                        let body = resp.text().await.unwrap_or_default();
                        return Err(ApiError::Authentication(body));
                    }

                    if status.is_server_error() && attempt < max_retries {
                        let delay = self.backoff_delay(attempt);
                        warn!("server error {status}; retrying after {delay:?}");
                        sleep(delay).await;
                        attempt += 1;
                        continue;
                    }

                    let text = resp.text().await.unwrap_or_default();
                    let details = serde_json::from_str::<Value>(&text).ok();
                    return Err(ApiError::http(
                        status,
                        if text.is_empty() {
                            status
                                .canonical_reason()
                                .unwrap_or("unknown error")
                                .to_string()
                        } else {
                            text
                        },
                        details,
                    ));
                }
            }
        }
    }

    fn build_url(&self, path: &str) -> String {
        let trimmed_base = self.options.base_url.trim_end_matches('/');
        let trimmed_path = path.trim_start_matches('/');
        format!("{trimmed_base}/{trimmed_path}")
    }

    fn build_cache_key(method: &Method, path: &str, query_pairs: &[(String, String)]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(method.as_str());
        hasher.update("::");
        hasher.update(path);
        hasher.update("::");

        let mut sorted = query_pairs.to_vec();
        sorted.sort();
        if let Ok(serialized) = serde_json::to_string(&sorted) {
            hasher.update(serialized);
        }

        format!("{:x}", hasher.finalize())
    }

    async fn get_from_cache(&self, key: &str) -> Result<Option<Vec<u8>>, ApiError> {
        let now = Instant::now();
        if let Some(entry) = {
            let guard = self.memory_cache.read().await;
            guard.get(key).cloned()
        } && entry.expires_at > now
        {
            debug!("cache hit (memory) for {key}");
            return Ok(Some((*entry.value).clone()));
        }

        let path = self.cache_file_path(key);
        match fs::read(&path).await {
            Ok(bytes) => {
                match serde_json::from_slice::<DiskCacheEntry>(&bytes) {
                    Ok(entry) => {
                        let expires_at = UNIX_EPOCH + Duration::from_secs(entry.expires_at);
                        if SystemTime::now() <= expires_at {
                            match general_purpose::STANDARD.decode(entry.body) {
                                Ok(body) => {
                                    self.store_in_memory(key.to_string(), body.clone());
                                    return Ok(Some(body));
                                }
                                Err(err) => {
                                    warn!("failed to decode cache entry: {err}");
                                    fs::remove_file(&path).await.ok();
                                }
                            }
                        } else {
                            fs::remove_file(&path).await.ok();
                        }
                    }
                    Err(err) => {
                        warn!("failed to parse cache entry: {err}");
                        fs::remove_file(&path).await.ok();
                    }
                }
                Ok(None)
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(ApiError::Cache(err)),
        }
    }

    async fn write_cache(&self, key: &str, body: &[u8]) -> Result<(), ApiError> {
        self.store_in_memory(key.to_string(), body.to_vec());

        let expires_at = SystemTime::now()
            .checked_add(self.options.cache_ttl)
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map_or_else(
                || {
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                },
                |duration| duration.as_secs(),
            );

        let entry = DiskCacheEntry {
            expires_at,
            body: general_purpose::STANDARD.encode(body),
        };

        let path = self.cache_file_path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.ok();
        }
        let serialized = serde_json::to_vec(&entry)?;
        fs::write(path, serialized).await?;

        debug!("cached response for key {key}");
        Ok(())
    }

    fn store_in_memory(&self, key: String, body: Vec<u8>) {
        let entry = CacheEntry {
            expires_at: Instant::now() + self.options.cache_ttl,
            value: Arc::new(body),
        };
        let cache = self.memory_cache.clone();
        tokio::spawn(async move {
            let mut guard = cache.write().await;
            guard.insert(key, entry);
        });
    }

    fn cache_file_path(&self, key: &str) -> PathBuf {
        let mut filename = String::from(key);
        filename.push_str(".json");
        self.options.cache_dir.join(filename)
    }

    fn backoff_delay(&self, attempt: usize) -> Duration {
        let multiplier = 1u32
            .checked_shl(u32::try_from(attempt).unwrap_or(u32::MAX))
            .unwrap_or(1);
        self.options
            .retry_base_delay
            .checked_mul(multiplier)
            .unwrap_or(self.options.retry_base_delay)
    }

    fn parse_retry_after(headers: &HeaderMap) -> Option<Duration> {
        headers.get(RETRY_AFTER).and_then(|value| {
            value.to_str().ok().and_then(|retry| {
                if let Ok(seconds) = retry.parse::<f64>()
                    && seconds.is_finite()
                    && seconds >= 0.0
                {
                    return Some(Duration::from_secs_f64(seconds));
                }
                None
            })
        })
    }

    fn extract_rate_limit_headers(headers: &HeaderMap) -> Option<RateLimitInfo> {
        let limit = headers
            .get("X-RateLimit-Limit")
            .and_then(|value| value.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok());
        let remaining = headers
            .get("X-RateLimit-Remaining")
            .and_then(|value| value.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok());
        let reset = headers
            .get("X-RateLimit-Reset")
            .and_then(|value| value.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());
        let retry_after = Self::parse_retry_after(headers);

        if limit.is_none() && remaining.is_none() && reset.is_none() && retry_after.is_none() {
            None
        } else {
            Some(RateLimitInfo {
                limit,
                remaining,
                reset,
                retry_after,
            })
        }
    }
}

fn build_query_pairs(query: &[(&str, &str)]) -> Vec<(String, String)> {
    query
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

fn is_offset_expired(details: Option<&Value>, message: &str) -> bool {
    let matches = |text: &str| {
        let lowered = text.to_ascii_lowercase();
        lowered.contains("offset") && (lowered.contains("expired") || lowered.contains("invalid"))
    };

    if let Some(payload) = details
        && let Some(errors) = payload.get("errors").and_then(|v| v.as_array())
        && errors.iter().any(|err| {
            err.get("message")
                .and_then(Value::as_str)
                .is_some_and(matches)
        })
    {
        return true;
    }

    matches(message)
}

fn validate_response_schema(value: &Value) -> Result<(), ApiError> {
    if let Value::Object(obj) = value {
        if obj.contains_key("data") || obj.contains_key("errors") {
            return Ok(());
        }
        Err(ApiError::Other(
            "response missing required `data` or `errors` field".to_string(),
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::auth::AuthToken;
    use futures_util::StreamExt;
    use mockito::{Matcher, Server};
    use secrecy::SecretString;
    use tempfile::TempDir;

    #[tokio::test]
    async fn get_current_user_fetches_data() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/users/me")
            .match_header("authorization", "Bearer test-token")
            .with_status(200)
            .with_body(r#"{ "data": { "name": "Test User" } }"#)
            .create_async()
            .await;

        let tmp = TempDir::new().unwrap();
        let token = AuthToken::new(SecretString::new("test-token".into()));
        let base_url = server.url();
        let client = ApiClient::builder(token)
            .base_url(base_url)
            .cache_dir(tmp.path().join("cache"))
            .build()
            .unwrap();

        let user: Value = client.get_current_user().await.unwrap();
        assert_eq!(user["data"]["name"], "Test User");
        drop(server);
    }

    #[tokio::test]
    async fn rate_limit_retries_then_succeeds() {
        let mut server = Server::new_async().await;
        let _m1 = server
            .mock("GET", "/users/me")
            .with_status(429)
            .with_header("Retry-After", "0.1")
            .create_async()
            .await;
        let _m2 = server
            .mock("GET", "/users/me")
            .with_status(200)
            .with_body(r#"{ "data": { "name": "Retry User" } }"#)
            .create_async()
            .await;

        let tmp = TempDir::new().unwrap();
        let token = AuthToken::new(SecretString::new("retry-token".into()));
        let base_url = server.url();
        let client = ApiClient::builder(token)
            .base_url(base_url)
            .cache_dir(tmp.path().join("cache"))
            .retry_base_delay(Duration::from_millis(50))
            .max_retries(2)
            .build()
            .unwrap();

        let user: Value = client.get_current_user().await.unwrap();
        assert_eq!(user["data"]["name"], "Retry User");
        drop(server);
    }

    #[tokio::test]
    async fn offline_uses_cache() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/users/me")
            .with_status(200)
            .with_body(r#"{ "data": { "name": "Cached User" } }"#)
            .create_async()
            .await;

        let tmp = TempDir::new().unwrap();
        let token = AuthToken::new(SecretString::new("cache-token".into()));
        let url = server.url();
        let client = ApiClient::builder(token)
            .base_url(url)
            .cache_dir(tmp.path().join("cache"))
            .cache_ttl(Duration::from_secs(60))
            .build()
            .unwrap();

        let user: Value = client.get_current_user().await.unwrap();
        assert_eq!(user["data"]["name"], "Cached User");

        client.set_offline(true);

        // Drop all mocks to ensure no HTTP requests are made.
        server.reset();
        drop(server);

        let cached: Value = client.get_current_user().await.unwrap();
        assert_eq!(cached["data"]["name"], "Cached User");
    }

    #[tokio::test]
    async fn rate_limit_headers_captured_on_success() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/users/me")
            .with_status(200)
            .with_header("X-RateLimit-Limit", "150")
            .with_header("X-RateLimit-Remaining", "149")
            .with_header("X-RateLimit-Reset", "1234567890")
            .with_body(r#"{ "data": { "name": "Metrics User" } }"#)
            .create_async()
            .await;

        let tmp = TempDir::new().unwrap();
        let token = AuthToken::new(SecretString::new("metrics-token".into()));
        let base_url = server.url();
        let client = ApiClient::builder(token)
            .base_url(base_url)
            .cache_dir(tmp.path().join("cache"))
            .build()
            .unwrap();

        client.get_current_user().await.unwrap();
        let info = client.rate_limit_info().await.expect("rate limit info");
        assert_eq!(info.limit, Some(150));
        assert_eq!(info.remaining, Some(149));
        assert_eq!(info.reset, Some(1_234_567_890));
        assert!(info.retry_after.is_none());
        drop(server);
    }

    #[tokio::test]
    async fn rate_limit_headers_captured_on_429() {
        let mut server = Server::new_async().await;
        let _m1 = server
            .mock("GET", "/users/me")
            .with_status(429)
            .with_header("Retry-After", "0.1")
            .with_header("X-RateLimit-Limit", "150")
            .with_header("X-RateLimit-Remaining", "0")
            .with_body("rate limited")
            .create_async()
            .await;
        let _m2 = server
            .mock("GET", "/users/me")
            .with_status(200)
            .with_body(r#"{ "data": { "name": "Retry User" } }"#)
            .create_async()
            .await;

        let tmp = TempDir::new().unwrap();
        let token = AuthToken::new(SecretString::new("metrics-rate-limit".into()));
        let base_url = server.url();
        let client = ApiClient::builder(token)
            .base_url(base_url)
            .cache_dir(tmp.path().join("cache"))
            .retry_base_delay(Duration::from_millis(50))
            .max_retries(2)
            .build()
            .unwrap();

        client.get_current_user().await.unwrap();
        let info = client.rate_limit_info().await.expect("rate limit info");
        assert_eq!(info.limit, Some(150));
        assert_eq!(info.remaining, Some(0));
        assert!(info.retry_after.is_some());
        drop(server);
    }

    #[tokio::test]
    async fn paginate_respects_manual_limit() {
        let mut server = Server::new_async().await;
        let _first = server
            .mock("GET", "/items")
            .with_status(200)
            .with_body(
                r#"{
                    "data": [ { "gid": "1" } ],
                    "next_page": { "offset": "abc", "path": "/items" }
                }"#,
            )
            .create_async()
            .await;
        let _second = server
            .mock("GET", "/items")
            .match_query(Matcher::UrlEncoded("offset".into(), "abc".into()))
            .with_status(200)
            .with_body(r#"{ "data": [ { "gid": "2" } ] }"#)
            .expect(0)
            .create_async()
            .await;

        let tmp = TempDir::new().unwrap();
        let token = AuthToken::new(SecretString::new("paginate-limit".into()));
        let base_url = server.url();
        let client = ApiClient::builder(token)
            .base_url(base_url)
            .cache_dir(tmp.path().join("cache"))
            .build()
            .unwrap();

        let stream = client.paginate_with_limit::<Value>("/items", vec![], Some(1));
        tokio::pin!(stream);
        let mut ids = Vec::new();
        while let Some(page) = stream.next().await {
            let page = page.expect("page result");
            ids.extend(
                page.iter()
                    .filter_map(|item| item.get("gid"))
                    .filter_map(Value::as_str)
                    .map(ToString::to_string),
            );
        }
        assert_eq!(ids, vec!["1".to_string()]);
        drop(server);
    }

    #[tokio::test]
    async fn paginate_handles_offset_expired() {
        let mut server = Server::new_async().await;
        let _first = server
            .mock("GET", "/items")
            .with_status(200)
            .with_body(
                r#"{
                    "data": [ { "gid": "1" } ],
                    "next_page": { "offset": "abc", "path": "/items" }
                }"#,
            )
            .create_async()
            .await;
        let _second = server
            .mock("GET", "/items")
            .match_query(Matcher::UrlEncoded("offset".into(), "abc".into()))
            .with_status(400)
            .with_body(
                r#"{
                    "errors": [ { "message": "offset is invalid or expired" } ]
                }"#,
            )
            .create_async()
            .await;

        let tmp = TempDir::new().unwrap();
        let token = AuthToken::new(SecretString::new("paginate-offset".into()));
        let base_url = server.url();
        let client = ApiClient::builder(token)
            .base_url(base_url)
            .cache_dir(tmp.path().join("cache"))
            .build()
            .unwrap();

        let stream = client.paginate::<Value>("/items", vec![]);
        tokio::pin!(stream);
        let mut ids = Vec::new();
        while let Some(page) = stream.next().await {
            let page = page.expect("page result");
            ids.extend(
                page.iter()
                    .filter_map(|item| item.get("gid"))
                    .filter_map(Value::as_str)
                    .map(ToString::to_string),
            );
        }
        assert_eq!(ids, vec!["1".to_string()]);
        drop(server);
    }

    #[tokio::test]
    async fn empty_response_returns_error() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/users/me")
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let tmp = TempDir::new().unwrap();
        let token = AuthToken::new(SecretString::new("empty-response".into()));
        let url = server.url();
        let client = ApiClient::builder(token)
            .base_url(url)
            .cache_dir(tmp.path().join("cache"))
            .build()
            .unwrap();

        let err = client.get_current_user().await.expect_err("should error");
        assert!(matches!(err, ApiError::Other(message) if message.contains("empty response")));
        drop(server);
    }

    #[tokio::test]
    async fn response_missing_data_field_errors() {
        let mut server = Server::new_async().await;
        let _m = server
            .mock("GET", "/users/me")
            .with_status(200)
            .with_body(r#"{ "foo": "bar" }"#)
            .create_async()
            .await;

        let tmp = TempDir::new().unwrap();
        let token = AuthToken::new(SecretString::new("missing-data".into()));
        let url = server.url();
        let client = ApiClient::builder(token)
            .base_url(url)
            .cache_dir(tmp.path().join("cache"))
            .build()
            .unwrap();

        let err = client.get_current_user().await.expect_err("should error");
        assert!(matches!(err, ApiError::Other(message) if message.contains("data")));
        drop(server);
    }
}
