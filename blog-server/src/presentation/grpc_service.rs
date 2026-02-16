use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::{
    application::{AuthService, BlogService},
    domain::{CreatePostDto, DomainError, LoginDto, RegisterUserDto, UpdatePostDto},
    infrastructure::JwtService,
};

// Include generated protobuf code
pub mod blog {
    tonic::include_proto!("blog");
}

use blog::{
    blog_service_server::BlogService as BlogServiceTrait, AuthResponse, CreatePostRequest,
    DeletePostRequest, DeletePostResponse, GetPostRequest, ListPostsRequest, ListPostsResponse,
    LoginRequest, Post, PostResponse, RegisterRequest, UpdatePostRequest, User,
};

pub struct BlogGrpcService {
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
    jwt_service: Arc<JwtService>,
}

impl BlogGrpcService {
    pub fn new(
        auth_service: Arc<AuthService>,
        blog_service: Arc<BlogService>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            auth_service,
            blog_service,
            jwt_service,
        }
    }

    fn extract_token(&self, request: &Request<impl std::fmt::Debug>) -> Result<i64, Status> {
        let token = request
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| Status::unauthenticated("Missing or invalid authorization header"))?;

        let claims = self
            .jwt_service
            .verify_token(token)
            .map_err(|_| Status::unauthenticated("Invalid token"))?;

        Ok(claims.user_id)
    }
}

#[tonic::async_trait]
impl BlogServiceTrait for BlogGrpcService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();

        let dto = RegisterUserDto {
            username: req.username,
            email: req.email,
            password: req.password,
        };

        match self.auth_service.register(dto).await {
            Ok((token, user)) => Ok(Response::new(AuthResponse {
                token,
                user: Some(User {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    created_at: user.created_at.to_rfc3339(),
                }),
            })),
            Err(DomainError::UserAlreadyExists) => Err(Status::already_exists("User already exists")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();

        let dto = LoginDto {
            username: req.username,
            password: req.password,
        };

        match self.auth_service.login(dto).await {
            Ok((token, user)) => Ok(Response::new(AuthResponse {
                token,
                user: Some(User {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    created_at: user.created_at.to_rfc3339(),
                }),
            })),
            Err(DomainError::InvalidCredentials) => Err(Status::unauthenticated("Invalid credentials")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let user_id = self.extract_token(&request)?;
        let req = request.into_inner();

        let dto = CreatePostDto {
            title: req.title,
            content: req.content,
        };

        match self.blog_service.create_post(dto, user_id).await {
            Ok(post) => Ok(Response::new(PostResponse {
                post: Some(Post {
                    id: post.id,
                    title: post.title,
                    content: post.content,
                    author_id: post.author_id,
                    created_at: post.created_at.to_rfc3339(),
                    updated_at: post.updated_at.to_rfc3339(),
                }),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_post(
        &self,
        request: Request<GetPostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let req = request.into_inner();

        match self.blog_service.get_post(req.id).await {
            Ok(post) => Ok(Response::new(PostResponse {
                post: Some(Post {
                    id: post.id,
                    title: post.title,
                    content: post.content,
                    author_id: post.author_id,
                    created_at: post.created_at.to_rfc3339(),
                    updated_at: post.updated_at.to_rfc3339(),
                }),
            })),
            Err(DomainError::PostNotFound) => Err(Status::not_found("Post not found")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn update_post(
        &self,
        request: Request<UpdatePostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let user_id = self.extract_token(&request)?;
        let req = request.into_inner();

        let dto = UpdatePostDto {
            title: req.title,
            content: req.content,
        };

        match self.blog_service.update_post(req.id, dto, user_id).await {
            Ok(post) => Ok(Response::new(PostResponse {
                post: Some(Post {
                    id: post.id,
                    title: post.title,
                    content: post.content,
                    author_id: post.author_id,
                    created_at: post.created_at.to_rfc3339(),
                    updated_at: post.updated_at.to_rfc3339(),
                }),
            })),
            Err(DomainError::PostNotFound) => Err(Status::not_found("Post not found")),
            Err(DomainError::Forbidden) => Err(Status::permission_denied("Forbidden")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<DeletePostResponse>, Status> {
        let user_id = self.extract_token(&request)?;
        let req = request.into_inner();

        match self.blog_service.delete_post(req.id, user_id).await {
            Ok(_) => Ok(Response::new(DeletePostResponse { success: true })),
            Err(DomainError::PostNotFound) => Err(Status::not_found("Post not found")),
            Err(DomainError::Forbidden) => Err(Status::permission_denied("Forbidden")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>, Status> {
        let req = request.into_inner();
        let limit = if req.limit == 0 { 10 } else { req.limit.min(100) } as i64;
        let offset = req.offset.max(0) as i64;

        match self.blog_service.list_posts(limit, offset).await {
            Ok((posts, total)) => Ok(Response::new(ListPostsResponse {
                posts: posts
                    .into_iter()
                    .map(|post| Post {
                        id: post.id,
                        title: post.title,
                        content: post.content,
                        author_id: post.author_id,
                        created_at: post.created_at.to_rfc3339(),
                        updated_at: post.updated_at.to_rfc3339(),
                    })
                    .collect(),
                total,
                limit: req.limit,
                offset: req.offset,
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
