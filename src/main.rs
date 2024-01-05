use std::{env, fmt::Debug};

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

#[derive(ValueEnum, Debug, Clone)]
enum Command {
    Add,    //C
    List,   //R
    Update, //U
    Delete, //D
}

/// Simple program to track todo items
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the command
    command: Command,
    // Todo title
    #[arg(short, long)]
    title: Option<String>,
    // Todo message
    #[arg(short, long)]
    message: Option<String>,
}

impl Args {
    fn args_check(&self) -> Option<(&String, &String)> {
        let Some(title) = &self.title else {
            println!("--title is required");
            return None;
        };
        let Some(message) = &self.message else {
            println!("--message is required");
            return None;
        };

        Some((title, message))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Todo {
    title: String,
    msg: String,
}

impl Todo {
    async fn add(conn: SqlitePool, todo: Todo) -> Result<(), ()> {
        let result = sqlx::query!(
            "insert into todos (title, msg) values ($1, $2)",
            todo.title,
            todo.msg
        )
        .execute(&conn)
        .await;
        if result.unwrap().rows_affected() != 1 {
            return Err(());
        }
        Ok(())
    }
    async fn list(conn: SqlitePool) -> Result<(), ()> {
        let result = sqlx::query_as!(Todo, "select * from todos")
            .fetch_all(&conn)
            .await;
        if let Err(_) = result {
            println!("error reading from todos");
            return Err(());
        }
        for r in result.unwrap() {
            println!("{} - {}", r.title, r.msg);
        }
        Ok(())
    }
    async fn update(conn: SqlitePool, todo: Todo) -> Result<(), ()> {
        let result = sqlx::query!(
            "update todos set title = $1, msg = $2 where title = $1",
            todo.title,
            todo.msg
        )
        .execute(&conn)
        .await;
        if result.unwrap().rows_affected() != 1 {
            return Err(());
        }
        Ok(())
    }
    async fn delete(conn: SqlitePool, title: &String) -> Result<(), ()> {
        let result = sqlx::query!("delete from todos where title = $1", title)
            .execute(&conn)
            .await;
        if result.unwrap().rows_affected() != 1 {
            return Err(());
        }
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv::dotenv().ok();
    let args = Args::parse();

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").expect("DATABASE_URL missing"))
        .await;

    let Ok(pool) = pool else {
        eprintln!("cannot connect to db");
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Migrations failed to run");

    match args.command {
        Command::Add => {
            let Some(args) = args.args_check() else {
                return;
            };
            Todo::add(
                pool,
                Todo {
                    title: args.0.to_string(),
                    msg: args.1.to_string(),
                },
            )
            .await
            .unwrap();
        }
        Command::List => {
            Todo::list(pool).await.unwrap();
        }
        Command::Update => {
            let Some(args) = args.args_check() else {
                return;
            };
            let title = args.0;
            let message = args.1;
            Todo::update(
                pool,
                Todo {
                    title: title.to_string(),
                    msg: message.to_string(),
                },
            )
            .await
            .unwrap();
        }
        Command::Delete => {
            let Some(title) = args.title else {
                println!("--title is required");
                return;
            };
            Todo::delete(pool, &title).await.unwrap();
        }
    }
}
