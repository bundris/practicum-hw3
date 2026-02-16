use tonic::{metadata::MetadataValue, Request};

use crate::error::BlogClientError;
use crate::{Post, User};

// Include generated protobuf code
pub mod blog {
    tonic::include_proto!("blog");
}

use blog::{
    blog_service_client::BlogServiceClient, CreatePostRequest, DeletePostRequest, GetPostRequest,
    ListPostsRequest, LoginRequest, RegisterRequest, UpdatePostRequest,
};

pub struct GrpcClient {
    client: BlogServiceClient<tonic::transport::Channel>,
}

impl GrpcClient {
    pub async fn new(addr: String) -> Result<Self, BlogClientError> {
        let client = BlogServiceClient::connect(addr).await?;
        Ok(Self { client })
    }

    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<(String, User), BlogClientError> {
        let request = Request::new(RegisterRequest {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        });

        let response = self.client.register(request).await?;
        let auth_response = response.into_inner();

        let user = auth_response.user.ok_or_else(|| {
            BlogClientError::InternalError("No user in response".to_string())
        })?;

        Ok((auth_response.token, map_user(user)))
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<(String, User), BlogClientError> {
        let request = Request::new(LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        });

        let response = self.client.login(request).await?;
        let auth_response = response.into_inner();

        let user = auth_response.user.ok_or_else(|| {
            BlogClientError::InternalError("No user in response".to_string())
        })?;

        Ok((auth_response.token, map_user(user)))
    }

    pub async fn create_post(
        &mut self,
        token: &str,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let mut request = Request::new(CreatePostRequest {
            title: title.to_string(),
            content: content.to_string(),
        });

        let bearer_token = format!("Bearer {}", token);
        request.metadata_mut().insert(
            "authorization",
            MetadataValue::try_from(&bearer_token)
                .map_err(|e| BlogClientError::InternalError(format!("Invalid token: {}", e)))?,
        );

        let response = self.client.create_post(request).await?;
        let post_response = response.into_inner();

        let post = post_response.post.ok_or_else(|| {
            BlogClientError::InternalError("No post in response".to_string())
        })?;

        Ok(map_post(post))
    }

    pub async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        let request = Request::new(GetPostRequest { id });

        let response = self.client.get_post(request).await.map_err(|status| {
            if status.code() == tonic::Code::NotFound {
                BlogClientError::NotFound
            } else {
                BlogClientError::GrpcError(status)
            }
        })?;

        let post_response = response.into_inner();

        let post = post_response.post.ok_or_else(|| {
            BlogClientError::InternalError("No post in response".to_string())
        })?;

        Ok(map_post(post))
    }

    pub async fn update_post(
        &mut self,
        token: &str,
        id: i64,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let mut request = Request::new(UpdatePostRequest {
            id,
            title: title.to_string(),
            content: content.to_string(),
        });

        let bearer_token = format!("Bearer {}", token);
        request.metadata_mut().insert(
            "authorization",
            MetadataValue::try_from(&bearer_token)
                .map_err(|e| BlogClientError::InternalError(format!("Invalid token: {}", e)))?,
        );

        let response = self.client.update_post(request).await.map_err(|status| {
            if status.code() == tonic::Code::NotFound {
                BlogClientError::NotFound
            } else if status.code() == tonic::Code::Unauthenticated {
                BlogClientError::Unauthorized
            } else {
                BlogClientError::GrpcError(status)
            }
        })?;

        let post_response = response.into_inner();

        let post = post_response.post.ok_or_else(|| {
            BlogClientError::InternalError("No post in response".to_string())
        })?;

        Ok(map_post(post))
    }

    pub async fn delete_post(&mut self, token: &str, id: i64) -> Result<(), BlogClientError> {
        let mut request = Request::new(DeletePostRequest { id });

        let bearer_token = format!("Bearer {}", token);
        request.metadata_mut().insert(
            "authorization",
            MetadataValue::try_from(&bearer_token)
                .map_err(|e| BlogClientError::InternalError(format!("Invalid token: {}", e)))?,
        );

        self.client.delete_post(request).await.map_err(|status| {
            if status.code() == tonic::Code::NotFound {
                BlogClientError::NotFound
            } else if status.code() == tonic::Code::Unauthenticated {
                BlogClientError::Unauthorized
            } else {
                BlogClientError::GrpcError(status)
            }
        })?;

        Ok(())
    }

    pub async fn list_posts(&mut self, limit: i32, offset: i32) -> Result<Vec<Post>, BlogClientError> {
        let request = Request::new(ListPostsRequest { limit, offset });

        let response = self.client.list_posts(request).await?;
        let list_response = response.into_inner();

        Ok(list_response.posts.into_iter().map(map_post).collect())
    }
}

fn map_user(user: blog::User) -> User {
    User {
        id: user.id,
        username: user.username,
        email: user.email,
        created_at: user.created_at,
    }
}

fn map_post(post: blog::Post) -> Post {
    Post {
        id: post.id,
        title: post.title,
        content: post.content,
        author_id: post.author_id,
        created_at: post.created_at,
        updated_at: post.updated_at,
    }
}
