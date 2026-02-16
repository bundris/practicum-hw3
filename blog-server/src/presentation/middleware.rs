use std::sync::Arc;

use actix_web::{
    dev::ServiceRequest,
    error::{ErrorInternalServerError, ErrorUnauthorized},
    web, Error, HttpMessage,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::infrastructure::JwtService;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub username: String,
}

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt_service = match req.app_data::<web::Data<Arc<JwtService>>>() {
        Some(service) => service.get_ref().clone(),
        None => {
            return Err((
                ErrorInternalServerError("JWT service not found"),
                req,
            ));
        }
    };

    let token = credentials.token();

    match jwt_service.verify_token(token) {
        Ok(claims) => {
            let user = AuthenticatedUser {
                user_id: claims.user_id,
                username: claims.username,
            };
            req.extensions_mut().insert(user);
            Ok(req)
        }
        Err(_) => Err((ErrorUnauthorized("Invalid token"), req)),
    }
}
