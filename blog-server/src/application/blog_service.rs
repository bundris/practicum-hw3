use std::sync::Arc;

use crate::{
    data::PostgresPostRepository,
    domain::{CreatePostDto, DomainError, Post, UpdatePostDto},
};

pub struct BlogService {
    post_repository: Arc<PostgresPostRepository>,
}

impl BlogService {
    pub fn new(post_repository: Arc<PostgresPostRepository>) -> Self {
        Self { post_repository }
    }

    pub async fn create_post(&self, dto: CreatePostDto, author_id: i64) -> Result<Post, DomainError> {
        self.post_repository
            .create_post(&dto.title, &dto.content, author_id)
            .await
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, DomainError> {
        self.post_repository
            .find_by_id(id)
            .await?
            .ok_or(DomainError::PostNotFound)
    }

    pub async fn update_post(
        &self,
        id: i64,
        dto: UpdatePostDto,
        user_id: i64,
    ) -> Result<Post, DomainError> {
        let post = self.get_post(id).await?;

        if post.author_id != user_id {
            return Err(DomainError::Forbidden);
        }

        self.post_repository
            .update_post(id, &dto.title, &dto.content)
            .await
    }

    pub async fn delete_post(&self, id: i64, user_id: i64) -> Result<(), DomainError> {
        let post = self.get_post(id).await?;

        if post.author_id != user_id {
            return Err(DomainError::Forbidden);
        }

        self.post_repository.delete_post(id).await?;
        Ok(())
    }

    pub async fn list_posts(&self, limit: i64, offset: i64) -> Result<(Vec<Post>, i64), DomainError> {
        let posts = self.post_repository.list_posts(limit, offset).await?;
        let total = self.post_repository.count_posts().await?;
        Ok((posts, total))
    }
}
