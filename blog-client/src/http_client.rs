use reqwest::Client;
use serde::Deserialize;

use crate::error::BlogClientError;
use crate::{AuthResponse, Post, User};

#[derive(Debug, Deserialize)]
struct PostsListResponse {
    posts: Vec<Post>,
    #[allow(dead_code)]
    total: i64,
    #[allow(dead_code)]
    limit: i32,
    #[allow(dead_code)]
    offset: i32,
}

pub struct HttpClient {
    base_url: String,
    client: Client,
}

impl HttpClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }

    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<(String, User), BlogClientError> {
        let response = self
            .client
            .post(format!("{}/api/auth/register", self.base_url))
            .json(&serde_json::json!({
                "username": username,
                "email": email,
                "password": password,
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let auth_response: AuthResponse = response.json().await?;
            Ok((auth_response.token, auth_response.user))
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(BlogClientError::InvalidRequest(format!(
                "Registration failed: {} - {}",
                status, error_text
            )))
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<(String, User), BlogClientError> {
        let response = self
            .client
            .post(format!("{}/api/auth/login", self.base_url))
            .json(&serde_json::json!({
                "username": username,
                "password": password,
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let auth_response: AuthResponse = response.json().await?;
            Ok((auth_response.token, auth_response.user))
        } else if response.status() == 401 {
            Err(BlogClientError::Unauthorized)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(BlogClientError::InvalidRequest(error_text))
        }
    }

    pub async fn create_post(
        &self,
        token: &str,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let response = self
            .client
            .post(format!("{}/api/posts", self.base_url))
            .bearer_auth(token)
            .json(&serde_json::json!({
                "title": title,
                "content": content,
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let post: Post = response.json().await?;
            Ok(post)
        } else if response.status() == 401 {
            Err(BlogClientError::Unauthorized)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(BlogClientError::InvalidRequest(error_text))
        }
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, BlogClientError> {
        let response = self
            .client
            .get(format!("{}/api/posts/{}", self.base_url, id))
            .send()
            .await?;

        if response.status().is_success() {
            let post: Post = response.json().await?;
            Ok(post)
        } else if response.status() == 404 {
            Err(BlogClientError::NotFound)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(BlogClientError::InvalidRequest(error_text))
        }
    }

    pub async fn update_post(
        &self,
        token: &str,
        id: i64,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let response = self
            .client
            .put(format!("{}/api/posts/{}", self.base_url, id))
            .bearer_auth(token)
            .json(&serde_json::json!({
                "title": title,
                "content": content,
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let post: Post = response.json().await?;
            Ok(post)
        } else if response.status() == 401 {
            Err(BlogClientError::Unauthorized)
        } else if response.status() == 404 {
            Err(BlogClientError::NotFound)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(BlogClientError::InvalidRequest(error_text))
        }
    }

    pub async fn delete_post(&self, token: &str, id: i64) -> Result<(), BlogClientError> {
        let response = self
            .client
            .delete(format!("{}/api/posts/{}", self.base_url, id))
            .bearer_auth(token)
            .send()
            .await?;

        if response.status().is_success() || response.status() == 204 {
            Ok(())
        } else if response.status() == 401 {
            Err(BlogClientError::Unauthorized)
        } else if response.status() == 404 {
            Err(BlogClientError::NotFound)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(BlogClientError::InvalidRequest(error_text))
        }
    }

    pub async fn list_posts(&self, limit: i32, offset: i32) -> Result<Vec<Post>, BlogClientError> {
        let response = self
            .client
            .get(format!("{}/api/posts", self.base_url))
            .query(&[("limit", limit), ("offset", offset)])
            .send()
            .await?;

        if response.status().is_success() {
            let posts_response: PostsListResponse = response.json().await?;
            Ok(posts_response.posts)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(BlogClientError::InvalidRequest(error_text))
        }
    }
}
