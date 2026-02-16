#[cfg(feature = "client")]
pub mod error;
#[cfg(feature = "grpc")]
pub mod grpc_client;
#[cfg(feature = "http")]
pub mod http_client;

use std::fmt;

use serde::{Deserialize, Serialize};

#[cfg(feature = "client")]
pub use crate::error::BlogClientError;
#[cfg(feature = "client")]
use crate::grpc_client::GrpcClient;
#[cfg(feature = "client")]
use crate::http_client::HttpClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "User #{}", self.id)?;
        writeln!(f, "Username: {}", self.username)?;
        writeln!(f, "Email: {}", self.email)?;
        write!(f, "Created: {}", self.created_at)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl fmt::Display for Post {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Post #{}", self.id)?;
        writeln!(f, "Title: {}", self.title)?;
        writeln!(f, "Content: {}", self.content)?;
        writeln!(f, "Author: {}", self.author_id)?;
        writeln!(f, "Created: {}", self.created_at)?;
        write!(f, "Updated: {}", self.updated_at)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[cfg(feature = "client")]
#[derive(Debug, Clone)]
pub enum Transport {
    Http(String),
    Grpc(String),
}

#[cfg(feature = "client")]
pub struct BlogClient {
    transport: Transport,
    http_client: Option<HttpClient>,
    grpc_client: Option<GrpcClient>,
    token: Option<String>,
}

#[cfg(feature = "client")]
impl BlogClient {
    pub async fn new(transport: Transport) -> Result<Self, BlogClientError> {
        match transport {
            Transport::Http(base_url) => {
                let http_client = HttpClient::new(base_url.clone());
                Ok(Self {
                    transport: Transport::Http(base_url),
                    http_client: Some(http_client),
                    grpc_client: None,
                    token: None,
                })
            }
            Transport::Grpc(addr) => {
                let grpc_client = GrpcClient::new(addr.clone()).await?;
                Ok(Self {
                    transport: Transport::Grpc(addr),
                    http_client: None,
                    grpc_client: Some(grpc_client),
                    token: None,
                })
            }
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<User, BlogClientError> {
        let (token, user) = match &self.transport {
            Transport::Http(_) => {
                let client = self
                    .http_client
                    .as_ref()
                    .ok_or_else(|| BlogClientError::InternalError("HTTP client missing".into()))?;
                client.register(username, email, password).await?
            }
            Transport::Grpc(_) => {
                let client = self
                    .grpc_client
                    .as_mut()
                    .ok_or_else(|| BlogClientError::InternalError("gRPC client missing".into()))?;
                client.register(username, email, password).await?
            }
        };

        self.token = Some(token);
        Ok(user)
    }

    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<User, BlogClientError> {
        let (token, user) = match &self.transport {
            Transport::Http(_) => {
                let client = self
                    .http_client
                    .as_ref()
                    .ok_or_else(|| BlogClientError::InternalError("HTTP client missing".into()))?;
                client.login(username, password).await?
            }
            Transport::Grpc(_) => {
                let client = self
                    .grpc_client
                    .as_mut()
                    .ok_or_else(|| BlogClientError::InternalError("gRPC client missing".into()))?;
                client.login(username, password).await?
            }
        };

        self.token = Some(token);
        Ok(user)
    }

    pub async fn create_post(&mut self, title: &str, content: &str) -> Result<Post, BlogClientError> {
        let token = self.token.as_deref().ok_or(BlogClientError::Unauthorized)?;

        match &self.transport {
            Transport::Http(_) => {
                let client = self
                    .http_client
                    .as_ref()
                    .ok_or_else(|| BlogClientError::InternalError("HTTP client missing".into()))?;
                client.create_post(token, title, content).await
            }
            Transport::Grpc(_) => {
                let client = self
                    .grpc_client
                    .as_mut()
                    .ok_or_else(|| BlogClientError::InternalError("gRPC client missing".into()))?;
                client.create_post(token, title, content).await
            }
        }
    }

    pub async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        match &self.transport {
            Transport::Http(_) => {
                let client = self
                    .http_client
                    .as_ref()
                    .ok_or_else(|| BlogClientError::InternalError("HTTP client missing".into()))?;
                client.get_post(id).await
            }
            Transport::Grpc(_) => {
                let client = self
                    .grpc_client
                    .as_mut()
                    .ok_or_else(|| BlogClientError::InternalError("gRPC client missing".into()))?;
                client.get_post(id).await
            }
        }
    }

    pub async fn update_post(
        &mut self,
        id: i64,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let token = self.token.as_deref().ok_or(BlogClientError::Unauthorized)?;

        match &self.transport {
            Transport::Http(_) => {
                let client = self
                    .http_client
                    .as_ref()
                    .ok_or_else(|| BlogClientError::InternalError("HTTP client missing".into()))?;
                client.update_post(token, id, title, content).await
            }
            Transport::Grpc(_) => {
                let client = self
                    .grpc_client
                    .as_mut()
                    .ok_or_else(|| BlogClientError::InternalError("gRPC client missing".into()))?;
                client.update_post(token, id, title, content).await
            }
        }
    }

    pub async fn delete_post(&mut self, id: i64) -> Result<(), BlogClientError> {
        let token = self.token.as_deref().ok_or(BlogClientError::Unauthorized)?;

        match &self.transport {
            Transport::Http(_) => {
                let client = self
                    .http_client
                    .as_ref()
                    .ok_or_else(|| BlogClientError::InternalError("HTTP client missing".into()))?;
                client.delete_post(token, id).await
            }
            Transport::Grpc(_) => {
                let client = self
                    .grpc_client
                    .as_mut()
                    .ok_or_else(|| BlogClientError::InternalError("gRPC client missing".into()))?;
                client.delete_post(token, id).await
            }
        }
    }

    pub async fn list_posts(&mut self, limit: i32, offset: i32) -> Result<Vec<Post>, BlogClientError> {
        match &self.transport {
            Transport::Http(_) => {
                let client = self
                    .http_client
                    .as_ref()
                    .ok_or_else(|| BlogClientError::InternalError("HTTP client missing".into()))?;
                client.list_posts(limit, offset).await
            }
            Transport::Grpc(_) => {
                let client = self
                    .grpc_client
                    .as_mut()
                    .ok_or_else(|| BlogClientError::InternalError("gRPC client missing".into()))?;
                client.list_posts(limit, offset).await
            }
        }
    }
}
