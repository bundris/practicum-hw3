use sqlx::{postgres::PgRow, PgPool, Row};

use crate::domain::{DomainError, User};

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, DomainError> {
        let user = sqlx::query(
            "INSERT INTO users (username, email, password_hash)
             VALUES ($1, $2, $3)
             RETURNING id, username, email, password_hash, created_at",
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .try_map(Self::parse_row)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError> {
        let user = sqlx::query(
            "SELECT id, username, email, password_hash, created_at
             FROM users
             WHERE username = $1",
        )
        .bind(username)
        .try_map(Self::parse_row)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn exists_by_username(&self, username: &str) -> Result<bool, DomainError> {
        let row = sqlx::query(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)"
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get(0))
    }

    pub async fn exists_by_email(&self, email: &str) -> Result<bool, DomainError> {
        let row = sqlx::query(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get(0))
    }
    fn parse_row(row: PgRow) -> Result<User, sqlx::Error> {
        Ok(User {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            email: row.try_get("email")?,
            password_hash: row.try_get("password_hash")?,
            created_at: row.try_get("created_at")?,
        })
    }
}
