use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use tonic::transport::Server;

mod application;
mod data;
mod domain;
mod infrastructure;
mod presentation;

use application::{AuthService, BlogService};
use data::{PostgresPostRepository, PostgresUserRepository};
use infrastructure::{create_pool, init_logging, run_migrations, JwtService};
use presentation::{grpc_service::blog::blog_service_server::BlogServiceServer, BlogGrpcService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    init_logging();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Database setup
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = create_pool(&database_url).await?;
    run_migrations(&pool).await?;

    tracing::info!("Database connected and migrations applied");

    // Initialize services
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let jwt_service = Arc::new(JwtService::new(&jwt_secret));

    let user_repository = Arc::new(PostgresUserRepository::new(pool.clone()));
    let post_repository = Arc::new(PostgresPostRepository::new(pool.clone()));

    let auth_service = Arc::new(AuthService::new(user_repository.clone(), jwt_service.clone()));
    let blog_service = Arc::new(BlogService::new(post_repository.clone()));

    // HTTP server setup
    let http_port = std::env::var("HTTP_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()?;

    let auth_service_http = auth_service.clone();
    let blog_service_http = blog_service.clone();
    let jwt_service_http = jwt_service.clone();

    let http_server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allow_any_header()
            .max_age(3600);

        let auth_middleware = HttpAuthentication::bearer(presentation::jwt_validator);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(auth_service_http.clone()))
            .app_data(web::Data::new(blog_service_http.clone()))
            .app_data(web::Data::new(jwt_service_http.clone()))
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(presentation::register))
                            .route("/login", web::post().to(presentation::login)),
                    )
                    .service(
                        web::scope("/posts")
                            .route("", web::get().to(presentation::list_posts))
                            .route("/{id}", web::get().to(presentation::get_post))
                            .service(
                                web::scope("")
                                    .wrap(auth_middleware)
                                    .route("", web::post().to(presentation::create_post))
                                    .route("/{id}", web::put().to(presentation::update_post))
                                    .route("/{id}", web::delete().to(presentation::delete_post)),
                            ),
                    ),
            )
    })
    .bind(("0.0.0.0", http_port))?
    .run();

    tracing::info!("HTTP server starting on 0.0.0.0:{}", http_port);

    // gRPC server setup
    let grpc_port = std::env::var("GRPC_PORT")
        .unwrap_or_else(|_| "50051".to_string())
        .parse::<u16>()?;

    let grpc_service = BlogGrpcService::new(auth_service, blog_service, jwt_service);
    let addr = format!("0.0.0.0:{}", grpc_port).parse()?;

    let grpc_server = Server::builder()
        .add_service(BlogServiceServer::new(grpc_service))
        .serve(addr);

    tracing::info!("gRPC server starting on 0.0.0.0:{}", grpc_port);

    // Run both servers concurrently
    tokio::select! {
        result = http_server => {
            result?;
        }
        result = grpc_server => {
            result?;
        }
    }

    Ok(())
}
