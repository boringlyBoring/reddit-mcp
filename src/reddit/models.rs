#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AccessTokenRequest {
    pub grant_type: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub expires_in: i32,
    pub scope: String,
    pub token_type: String,
}
