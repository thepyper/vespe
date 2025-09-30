use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use crate::error::ProjectError;
use super::LLMClient;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use oauth2::{
    AuthorizationCode,
    AuthUrl,
    ClientId,
    ClientSecret,
    CsrfToken,
    PkceCodeChallenge,
    RedirectUrl,
    Scope,
    TokenResponse,
    TokenUrl,
    RefreshToken,
	EndpointNotSet,
	EndpointSet,
};
use oauth2::StandardRevocableToken;
use oauth2::basic::{BasicClient, BasicTokenResponse, BasicErrorResponse, BasicTokenIntrospectionResponse, BasicRevocationErrorResponse };
use url::Url;

// --- Token Storage ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub created_at: DateTime<Utc>,
}

fn token_path(project_root: &PathBuf) -> PathBuf {
    project_root.join(".vespe").join("gemini_token.json")
}

fn save_token_to_disk(project_root: &PathBuf, token: &SerializableToken) -> Result<(), ProjectError> {
    let path = token_path(project_root);
    let json = serde_json::to_string_pretty(token)
        .map_err(|e| ProjectError::Json(e))?;
    fs::write(path, json).map_err(|e| ProjectError::Io(e))?;
    Ok(())
}

fn load_token_from_disk(project_root: &PathBuf) -> Option<SerializableToken> {
    let path = token_path(project_root);
    if !path.exists() {
        return None;
    }
    let json = fs::read_to_string(path).ok()?;
    serde_json::from_str(&json).ok()
}

async fn perform_oauth_flow(project_root: &PathBuf, client_id: String, client_secret: String) -> Result<SerializableToken, ProjectError> {
    let client = BasicClient::new(ClientId::new(client_id))
        .set_client_secret(ClientSecret::new(client_secret))
        .set_auth_uri(AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?)
        .set_token_uri(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?)
        .set_redirect_uri(RedirectUrl::new("http://127.0.0.1:8080".to_string())?);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("https://www.googleapis.com/auth/generative-language.retriever".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    println!("Please open the following URL in your browser to authenticate:");
    println!("\t{}", auth_url);

    webbrowser::open(auth_url.as_str()).map_err(|e| ProjectError::InvalidOperation(format!("Failed to open web browser: {}", e)))?;

    let server = tiny_http::Server::http("127.0.0.1:8080").unwrap();
    let request = server.recv().map_err(|e| ProjectError::InvalidOperation(format!("Failed to receive request: {}", e)))?;
    let url = Url::parse(&format!("http://127.0.0.1:8080{}", request.url())).unwrap();

    let code = url.query_pairs().find_map(|(key, value)| if key == "code" { Some(value.into_owned()) } else { None }).ok_or_else(|| ProjectError::InvalidOperation("Authorization code not found in redirect URL".to_string()))?;
    let state = url.query_pairs().find_map(|(key, value)| if key == "state" { Some(value.into_owned()) } else { None }).ok_or_else(|| ProjectError::InvalidOperation("State not found in redirect URL".to_string()))?;

    if state != *csrf_token.secret() {
        return Err(ProjectError::InvalidOperation("CSRF token mismatch".to_string()));
    }

    let http_client = reqwest::Client::new();
    let token: BasicTokenResponse = client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(&http_client)
        .await
        .map_err(|e| ProjectError::LLMClientError(format!("Failed to exchange code for token: {}", e)))?;

    let serializable_token = SerializableToken {
        access_token: token.access_token().secret().clone(),
        token_type: "Bearer".to_string(),
        expires_in: token.expires_in().map_or(0, |d| d.as_secs()),
        refresh_token: token.refresh_token().map(|t| t.secret().clone()),
        scope: token.scopes().map(|s| s.iter().map(|sc| sc.as_str()).collect::<Vec<_>>().join(" ")),
        created_at: Utc::now(),
    };

    save_token_to_disk(project_root, &serializable_token)?;

    let response = tiny_http::Response::from_string("Authentication successful! You can close this window now.");
    request.respond(response).map_err(|e| ProjectError::Io(e.into()))?;

    Ok(serializable_token)
}

pub struct GeminiClient {
    model: String,
    client: reqwest::Client,
    project_root: PathBuf,
    oauth_client: BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>,
    token: Arc<Mutex<SerializableToken>>,
}

impl GeminiClient {
    pub async fn new(project_root: PathBuf, model: String, client_id: String, client_secret: String) -> Result<Self, ProjectError> {
        let oauth_client = BasicClient::new(ClientId::new(client_id.clone()))
            .set_client_secret(ClientSecret::new(client_secret.clone()))
            .set_auth_uri(AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?)
            .set_token_uri(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?)
            .set_redirect_uri(RedirectUrl::new("http://127.0.0.1:8080".to_string())?);

        let token = match load_token_from_disk(&project_root) {
            Some(token) => token,
            None => perform_oauth_flow(&project_root, client_id, client_secret).await?,
        };

        Ok(GeminiClient {
            model,
            client: reqwest::Client::new(),
            project_root,
            oauth_client,
            token: Arc::new(Mutex::new(token)),
        })
    }
}

#[async_trait]
impl LLMClient for GeminiClient {
    async fn send_query(&self, formatted_prompt: String) -> Result<String, ProjectError> {
        let mut token_guard = self.token.lock().await;

        let expires_at = token_guard.created_at + chrono::Duration::seconds(token_guard.expires_in as i64);

        if Utc::now() >= expires_at {
            let refresh_token = token_guard.refresh_token.as_ref().unwrap();
            let new_token: BasicTokenResponse = self.oauth_client
                .exchange_refresh_token(&RefreshToken::new(refresh_token.clone()))
                .request_async(&reqwest::Client::new())
                .await
                .map_err(|e| ProjectError::LLMClientError(format!("Failed to refresh token: {}", e)))?;
            
            let new_serializable_token = SerializableToken {
                access_token: new_token.access_token().secret().clone(),
                token_type: "Bearer".to_string(),
                expires_in: new_token.expires_in().map_or(0, |d| d.as_secs()),
                refresh_token: new_token.refresh_token().map(|t| t.secret().clone()).or(token_guard.refresh_token.clone()),
                scope: new_token.scopes().map(|s| s.iter().map(|sc| sc.as_str()).collect::<Vec<_>>().join(" ")).or(token_guard.scope.clone()),
                created_at: Utc::now(),
            };

            save_token_to_disk(&self.project_root, &new_serializable_token)?;
            *token_guard = new_serializable_token;
        }

        let access_token = token_guard.access_token.clone();
        drop(token_guard);

        debug!("Gemini Request: Model={}", self.model);
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", self.model);
        let payload = serde_json::json!({
            "contents": [
                {"parts": [{"text": formatted_prompt}]}
            ]
        });

        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ProjectError::LLMClientError(format!("Gemini request failed: {}", e)))?;

        let response_json: serde_json::Value = response.json().await
            .map_err(|e| ProjectError::LLMClientError(format!("Failed to get Gemini JSON response: {}", e)))?;
        debug!("Gemini Response: {}", serde_json::to_string_pretty(&response_json).unwrap_or_else(|_| "<unparseable JSON>".to_string()));

        // Extract content from Gemini response
        response_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ProjectError::LLMClientError("Gemini response missing expected content.".to_string()))
    }
}
