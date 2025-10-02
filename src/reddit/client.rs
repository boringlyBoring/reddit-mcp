use anyhow::Result;
use dotenv::dotenv;
use reqwest::{Client, StatusCode, header};
use rmcp::{
    ServerHandler,
    model::{ServerCapabilities, ServerInfo},
    schemars, tool,
};
use std::env;
use uuid::Uuid;

use crate::reddit::models::{AccessTokenRequest, AccessTokenResponse, SearchSubredditNameRequest};

const AUTH_URL: &str = "https://www.reddit.com/api/v1/access_token";
const BASE_URL: &str = "https://oauth.reddit.com/api";
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

    async fn get_request<T, D>(
        &self,
        url: &str,
        auth_token: &str,
        json_data: D,
    ) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
        D: serde::Serialize,
    {
        tracing::info!("Making GET request to: {}", url);

        let headers = header::HeaderMap::new();

        let response = self
            .client
            .get(url)
            .headers(headers)
            .header(header::USER_AGENT, USER_AGENT)
            .header(header::AUTHORIZATION, auth_token)
            .query(&json_data)
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

    async fn post_request<T, D>(&self, url: &str, post_data: D) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
        D: serde::Serialize,
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

    #[tool(description = "Get access_token to authenticate from reddit")]
    async fn get_access_token(&self) -> String {
        tracing::info!("Calling /api/access_token to get Authorization token");

        let access_token_request = AccessTokenRequest {
            grant_type: "password".to_string(),
            username: self.username.clone(),
            password: self.password.clone(),
        };

        let access_token_response = self
            .post_request::<AccessTokenResponse, AccessTokenRequest>(
                &AUTH_URL,
                access_token_request,
            )
            .await;

        match access_token_response {
            Ok(token) => token.access_token,
            Err(e) => {
                tracing::error!("Failed to fetch the access token: {}", e);
                "Unable to fetch access_token from reddit".to_string()
            }
        }
    }

    #[tool(description = "List subreddit names that begin with a query string.")]
    async fn search_subreddit_names(
        &self,
        #[tool(param)]
        #[schemars(description = "Subreddits whose names begin with query will be returned")]
        query: String,
        #[tool(param)]
        #[schemars(description = "If exact is true, only an exact match will be returned.")]
        exact: bool,
        #[tool(param)]
        #[schemars(
            description = "If include_over_18 is false, subreddits with over-18 content restrictions will be filtered from the results."
        )]
        include_over_18: bool,
        #[tool(param)]
        #[schemars(
            description = "If include_unadvertisable is False, subreddits that have hide_ads set to True or are on the anti_ads_subreddits list will be filtered."
        )]
        include_unadvertisable: bool,
        #[tool(param)]
        #[schemars(description = "If type_ahead is False")]
        type_ahead: bool,
        #[tool(param)]
        #[schemars(
            description = "Access token from reddit access_token api to authenticate requests"
        )]
        access_token: String,
    ) -> Result<String, String> {
        tracing::info!("Calling /api/search_reddit_names.json");

        let url = format!("{}/search_reddit_names", BASE_URL);
        let uuid = Uuid::new_v4();
        let auth_token = format!("Bearer {}", access_token);

        let search_subreddit_names_request = SearchSubredditNameRequest {
            exact: exact,
            include_over_18: include_over_18,
            include_unadvertisable: include_unadvertisable,
            query: query,
            search_query_id: uuid.to_string(),
            typeahead_active: type_ahead,
        };

        let search_response = self
            .get_request::<String, SearchSubredditNameRequest>(
                &url,
                &auth_token,
                search_subreddit_names_request,
            )
            .await;
        search_response
    }
}

#[tool(tool_box)]
impl ServerHandler for RedditClient {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A MCP server for accessing Reddit".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
