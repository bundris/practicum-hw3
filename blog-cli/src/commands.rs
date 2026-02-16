use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
  Register {
    #[arg(long)]
    username: String,
    #[arg(long)]
    email: String,
    #[arg(long)]
    password: String,
  },
  Login {
    #[arg(long)]
    username: String,
    #[arg(long)]
    password: String,
  },
  Create {
    #[arg(long)]
    title: String,
    #[arg(long)]
    content: String,
  },
  Get {
    #[arg(long)]
    id: i64,
  },
  Update {
    #[arg(long)]
    id: i64,
    #[arg(long)]
    title: String,
    #[arg(long)]
    content: String,
  },
  Delete {
    #[arg(long)]
    id: i64,
  },
  List {
    #[arg(long, default_value = "10")]
    limit: i32,
    #[arg(long, default_value = "0")]
    offset: i32,
  },
}