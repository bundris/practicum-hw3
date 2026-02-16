use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::{
    data::PostgresUserRepository,
    domain::{DomainError, LoginDto, RegisterUserDto, User},
    infrastructure::JwtService,
};

pub struct AuthService {
    user_repository: Arc<PostgresUserRepository>,
    jwt_service: Arc<JwtService>,
}

impl AuthService {
    pub fn new(user_repository: Arc<PostgresUserRepository>, jwt_service: Arc<JwtService>) -> Self {
        Self {
            user_repository,
            jwt_service,
        }
    }

    pub async fn register(&self, dto: RegisterUserDto) -> Result<(String, User), DomainError> {
        // Check if user already exists
        if self
            .user_repository
            .exists_by_username(&dto.username)
            .await?
        {
            return Err(DomainError::UserAlreadyExists);
        }

        if self.user_repository.exists_by_email(&dto.email).await? {
            return Err(DomainError::UserAlreadyExists);
        }

        // Hash password
        let password_hash = self.hash_password(&dto.password)?;

        // Create user
        let user = self
            .user_repository
            .create_user(&dto.username, &dto.email, &password_hash)
            .await?;

        // Generate token
        let token = self.jwt_service.generate_token(user.id, &user.username)?;

        Ok((token, user))
    }

    pub async fn login(&self, dto: LoginDto) -> Result<(String, User), DomainError> {
        // Find user
        let user = self
            .user_repository
            .find_by_username(&dto.username)
            .await?
            .ok_or(DomainError::InvalidCredentials)?;

        // Verify password
        self.verify_password(&dto.password, &user.password_hash)?;

        // Generate token
        let token = self.jwt_service.generate_token(user.id, &user.username)?;

        Ok((token, user))
    }

    fn hash_password(&self, password: &str) -> Result<String, DomainError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| DomainError::InternalError(format!("Failed to hash password: {}", e)))?
            .to_string();
        Ok(password_hash)
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<(), DomainError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| DomainError::InternalError(format!("Failed to parse hash: {}", e)))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| DomainError::InvalidCredentials)
    }
}
