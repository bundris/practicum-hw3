mod commands;

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use blog_client::{BlogClient, Transport};
use clap::{Parser};
use crate::commands::Commands;

const TOKEN_FILE: &str = ".blog_token";
const DEFAULT_HTTP_SERVER: &str = "http://localhost:8080";
const DEFAULT_GRPC_SERVER: &str = "http://localhost:50051";

#[derive(Parser)]
#[command(name = "blog-cli")]
#[command(about = "A CLI for managing blog posts", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true)]
    grpc: bool,

    #[arg(long, global = true)]
    server: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let server_addr = cli.server.unwrap_or_else(|| {
        if cli.grpc {
            DEFAULT_GRPC_SERVER.to_string()
        } else {
            DEFAULT_HTTP_SERVER.to_string()
        }
    });

    let transport = if cli.grpc {
        Transport::Grpc(server_addr)
    } else {
        Transport::Http(server_addr)
    };

    let mut client = BlogClient::new(transport).await?;

    if let Ok(token) = load_token() {
        client.set_token(token);
    }

    // Execute command
    match cli.command {
        Commands::Register {
            username,
            email,
            password,
        } => {
            let user = client.register(&username, &email, &password).await?;
            if let Some(token) = client.get_token() {
                save_token(token)?;
                println!("Registration successful");
            }
            println!("{}", user);
        }
        Commands::Login { username, password } => {
            let user = client.login(&username, &password).await?;
            if let Some(token) = client.get_token() {
                save_token(token)?;
                println!("Login successful");
            }
            println!("{}", user);
        }
        Commands::Create { title, content } => {
            let post = client.create_post(&title, &content).await?;
            println!("Post created");
            println!("{}", post);
        }
        Commands::Get { id } => {
            let post = client.get_post(id).await?;
            println!("{}", post);
        }
        Commands::Update {
            id,
            title,
            content,
        } => {
            let post = client.update_post(id, &title, &content).await?;
            println!("Post updated");
            println!("{}", post);
        }
        Commands::Delete { id } => {
            client.delete_post(id).await?;
            println!("Post deleted successfully");
        }
        Commands::List { limit, offset } => {
            let posts = client.list_posts(limit, offset).await?;
            let count = posts.len();
            println!("Found {} post(s):", count);
            for post in &posts {
                println!("{}", post);
                println!();
            }
        }
    }

    Ok(())
}

fn load_token() -> Result<String> {
    let token_path = get_token_path()?;
    let token = fs::read_to_string(token_path)?;
    Ok(token.trim().to_string())
}

fn save_token(token: &str) -> Result<()> {
    let token_path = get_token_path()?;
    fs::write(token_path, token)?;
    Ok(())
}

fn get_token_path() -> Result<PathBuf> {
    Ok(PathBuf::from(TOKEN_FILE))
}
