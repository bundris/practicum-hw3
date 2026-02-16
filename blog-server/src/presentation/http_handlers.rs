use std::sync::Arc;

use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};

use crate::{
    application::{AuthService, BlogService},
    domain::{CreatePostDto, DomainError, LoginDto, Post, RegisterUserDto, UpdatePostDto, User},
    presentation::middleware::AuthenticatedUser,
};

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    user: UserResponse,
}

#[derive(Serialize)]
struct UserResponse {
    id: i64,
    username: String,
    email: String,
    created_at: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at.to_rfc3339(),
        }
    }
}

#[derive(Serialize)]
struct PostResponse {
    id: i64,
    title: String,
    content: String,
    author_id: i64,
    created_at: String,
    updated_at: String,
}

impl From<Post> for PostResponse {
    fn from(post: Post) -> Self {
        Self {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Serialize)]
struct PostsListResponse {
    posts: Vec<PostResponse>,
    total: i64,
    limit: i32,
    offset: i32,
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    limit: i32,
    #[serde(default)]
    offset: i32,
}

fn default_limit() -> i32 {
    10
}

// Auth endpoints
pub async fn register(
    auth_service: web::Data<Arc<AuthService>>,
    dto: web::Json<RegisterUserDto>,
) -> Result<HttpResponse> {
    match auth_service.register(dto.into_inner()).await {
        Ok((token, user)) => Ok(HttpResponse::Created().json(AuthResponse {
            token,
            user: user.into(),
        })),
        Err(DomainError::UserAlreadyExists) => Ok(HttpResponse::Conflict().json(serde_json::json!({
            "error": "User already exists"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))),
    }
}

pub async fn login(
    auth_service: web::Data<Arc<AuthService>>,
    dto: web::Json<LoginDto>,
) -> Result<HttpResponse> {
    match auth_service.login(dto.into_inner()).await {
        Ok((token, user)) => Ok(HttpResponse::Ok().json(AuthResponse {
            token,
            user: user.into(),
        })),
        Err(DomainError::InvalidCredentials) => Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid credentials"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))),
    }
}

// Post endpoints
pub async fn create_post(
    blog_service: web::Data<Arc<BlogService>>,
    dto: web::Json<CreatePostDto>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user = req
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;

    match blog_service.create_post(dto.into_inner(), user.user_id).await {
        Ok(post) => Ok(HttpResponse::Created().json(PostResponse::from(post))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))),
    }
}

pub async fn get_post(
    blog_service: web::Data<Arc<BlogService>>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let post_id = path.into_inner();

    match blog_service.get_post(post_id).await {
        Ok(post) => Ok(HttpResponse::Ok().json(PostResponse::from(post))),
        Err(DomainError::PostNotFound) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Post not found"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))),
    }
}

pub async fn update_post(
    blog_service: web::Data<Arc<BlogService>>,
    path: web::Path<i64>,
    dto: web::Json<UpdatePostDto>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user = req
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;

    let post_id = path.into_inner();

    match blog_service
        .update_post(post_id, dto.into_inner(), user.user_id)
        .await
    {
        Ok(post) => Ok(HttpResponse::Ok().json(PostResponse::from(post))),
        Err(DomainError::PostNotFound) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Post not found"
        }))),
        Err(DomainError::Forbidden) => Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Forbidden"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))),
    }
}

pub async fn delete_post(
    blog_service: web::Data<Arc<BlogService>>,
    path: web::Path<i64>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user = req
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;

    let post_id = path.into_inner();

    match blog_service.delete_post(post_id, user.user_id).await {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(DomainError::PostNotFound) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Post not found"
        }))),
        Err(DomainError::Forbidden) => Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Forbidden"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))),
    }
}

pub async fn list_posts(
    blog_service: web::Data<Arc<BlogService>>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse> {
    let limit = query.limit.max(1).min(100) as i64;
    let offset = query.offset.max(0) as i64;

    match blog_service.list_posts(limit, offset).await {
        Ok((posts, total)) => Ok(HttpResponse::Ok().json(PostsListResponse {
            posts: posts.into_iter().map(PostResponse::from).collect(),
            total,
            limit: query.limit,
            offset: query.offset,
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))),
    }
}
