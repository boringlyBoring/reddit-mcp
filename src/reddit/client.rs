use anyhow::Result;
use dotenv::dotenv;
use reqwest::{Client, StatusCode, header};
use rmcp::tool;
use std::collections::HashMap;
use std::env;

const BASE_URL: &str = "https://www.reddit.com/api/v1/access_token";
const AUTH_URL: &str = "{}/authorize?client_id={}&response_type=code&
    state=RANDOM_STRING&redirect_uri=URI&duration=DURATION&scope=SCOPE_STRING";
const USER_AGENT: &str = "reddit:mcp:v1 (by /u/boringly_boring)";

#[derive(Debug, Clone)]
pub struct RedditClient {
    client: Client,
    client_id: String,
    client_secret: String,
    username: String,
    password: String,
    redirect_url: String,
}

#[tool(tool_box)]
impl RedditClient {
    #[allow(dead_code)]
    pub fn new() -> Self {
        dotenv().ok();
        let client: Client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to create http client");

        let client_id: String = env::var("CLIENT_ID").expect("Expected Client Id");
        let client_secret: String = env::var("CLIENT_SECRET").expect("Excepted Client Secret");
        let username: String = env::var("REDDIT_USERNAME").expect("Expected Reddit Username");
        let password: String = env::var("REDDIT_PASSWORD").expect("Execpted Reddit Password");
        let redirect_url: String =
            env::var("REDIRECT_URL").expect("Exceped Redirect Url added during app registration");

        Self {
            client,
            client_id,
            client_secret,
            username,
            password,
            redirect_url,
        }
    }

    async fn get_request<T>(&self, url: &str) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        tracing::info!("Making GET request to: {}", url);

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        tracing::info!("Received response: {:?}", response);

        match response.status() {
            StatusCode::OK => response
                .json::<T>()
                .await
                .map_err(|e| format!("Failed to parse the response: {}", e)),
            status => Err(format!("Request failed with status: {}", status)),
        }
    }

    async fn post_request<T>(
        &self,
        url: &str,
        post_data: HashMap<String, String>,
    ) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        tracing::info!("Making POST request to: {}", url);

        let headers = header::HeaderMap::new();

        let response = self
            .client
            .post(url)
            .basic_auth(self.client_id.clone(), Some(self.client_secret.clone()))
            .headers(headers)
            .header(header::USER_AGENT, USER_AGENT)
            .form(&post_data)
            .send()
            .await
            .map_err(|e| format!("PSOT request failed: {}", e))?;

        tracing::info!("Received response: {:?}", response);

        match response.status() {
            StatusCode::OK => response
                .json::<T>()
                .await
                .map_err(|e| format!("Failed to parse the request: {}", e)),
            status => Err(format!("Request failed with status: {}", status)),
        }
    }
}
