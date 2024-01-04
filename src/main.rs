use std::{env, fmt::Debug, fs};

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;

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
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TodoList(Vec<Todo>);

impl TodoList {
    fn add(&mut self, todo: Todo) {
        self.0.push(todo)
    }
    fn print(&self) {
        for t in &self.0 {
            println!("{} - {}", t.title, t.message)
        }
    }
    fn update(&mut self, todo: Todo) {
        let to_update = self.0.iter_mut().find(|t| t.title == todo.title);
        let Some(to_update) = to_update else {
            println!("cannot find a todo item with that title");
            return;
        };
        to_update.message = todo.message.to_string();
    }
    fn delete(&mut self, title: &String) {
        let delete_index = self.0.iter().position(|t| &t.title == title);
        let Some(delete_index) = delete_index else {
            println!("cannot find a todo item with that title");
            return;
        };
        self.0.remove(delete_index);
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    use sqlx::Connection;
    let conn = SqliteConnection::connect(&env::var("DATABASE_URL").unwrap()).await;
    let Ok(conn) = conn else {
        eprintln!("cannot connect to db");
        return;
    };

    let file_str = match fs::read_to_string("data.json") {
        Ok(file) => file,
        Err(_) => {
            let empty: Vec<Todo> = Vec::new();
            let json_str = serde_json::to_string_pretty(&empty).unwrap();
            fs::write("data.json", &json_str).unwrap();
            json_str
        }
    };

    let todo_list: Vec<Todo> = serde_json::from_str(&file_str).unwrap();
    let mut list = TodoList(todo_list);
    match args.command {
        Command::Add => {
            let Some(args) = args.args_check() else {
                return;
            };
            list.add(Todo {
                title: args.0.to_string(),
                message: args.1.to_string(),
            });
        }
        Command::List => list.print(),
        Command::Update => {
            let Some(args) = args.args_check() else {
                return;
            };
            let title = args.0;
            let message = args.1;
            list.update(Todo {
                title: title.to_string(),
                message: message.to_string(),
            })
        }
        Command::Delete => {
            let Some(title) = args.title else {
                println!("--title is required");
                return;
            };
            list.delete(&title);
        }
    }

    let json_str = serde_json::to_string_pretty(&list.0).unwrap();
    fs::write("data.json", &json_str).unwrap();
}
