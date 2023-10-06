use std::{fmt::Debug, fs};

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

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
}

fn main() {
    let args = Args::parse();
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
            if args.title.is_none() || args.message.is_none() {
                return;
            }
            list.add(Todo {
                title: args.title.unwrap(),
                message: args.message.unwrap(),
            });
        }
        Command::List => {
            for t in &list.0 {
                println!("{} - {}", t.title, t.message)
            }
        }
        Command::Update => {
            let Some(title) = args.title else  {
                println!("--title is required");
                return;
            };
            let Some(message) = args.message else  {
                println!("--message is required");
                return;
            };
            let todo = list.0.iter_mut().find(|t| t.title == title);
            let Some(todo) = todo else {
                println!("cannot find a todo item with that title");
                return;
            };
            todo.message = message;
        }
        Command::Delete => todo!(),
    }

    let json_str = serde_json::to_string_pretty(&list.0).unwrap();
    fs::write("data.json", &json_str).unwrap();
}
