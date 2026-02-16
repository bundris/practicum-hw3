use chrono::Utc;
use sqlx::{PgPool, Row};

use crate::domain::{DomainError, Post};

pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_post(
        &self,
        title: &str,
        content: &str,
        author_id: i64,
    ) -> Result<Post, DomainError> {
        let row = sqlx::query(
            "INSERT INTO posts (title, content, author_id)
             VALUES ($1, $2, $3)
             RETURNING id, title, content, author_id, created_at, updated_at"
        )
        .bind(title)
        .bind(content)
        .bind(author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Post {
            id: row.get("id"),
            title: row.get("title"),
            content: row.get("content"),
            author_id: row.get("author_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Post>, DomainError> {
        let row = sqlx::query(
            "SELECT id, title, content, author_id, created_at, updated_at
             FROM posts
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Post {
            id: r.get("id"),
            title: r.get("title"),
            content: r.get("content"),
            author_id: r.get("author_id"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    pub async fn update_post(
        &self,
        id: i64,
        title: &str,
        content: &str,
    ) -> Result<Post, DomainError> {
        let row = sqlx::query(
            "UPDATE posts
             SET title = $2, content = $3, updated_at = $4
             WHERE id = $1
             RETURNING id, title, content, author_id, created_at, updated_at"
        )
        .bind(id)
        .bind(title)
        .bind(content)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(Post {
            id: row.get("id"),
            title: row.get("title"),
            content: row.get("content"),
            author_id: row.get("author_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn delete_post(&self, id: i64) -> Result<bool, DomainError> {
        let result = sqlx::query(
            "DELETE FROM posts WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn list_posts(&self, limit: i64, offset: i64) -> Result<Vec<Post>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, title, content, author_id, created_at, updated_at
             FROM posts
             ORDER BY created_at DESC
             LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| Post {
            id: r.get("id"),
            title: r.get("title"),
            content: r.get("content"),
            author_id: r.get("author_id"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect())
    }

    pub async fn count_posts(&self) -> Result<i64, DomainError> {
        let row = sqlx::query(
            "SELECT COUNT(*) FROM posts"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get(0))
    }
}
