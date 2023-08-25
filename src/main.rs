use std::fmt::Debug;

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Debug, Clone)]
enum Command {
    Add,
    Remove,
    List,
}

/// Simple program to track todo items
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the command
    command: Command,
}

impl ToString for Command {
    fn to_string(&self) -> String {
        return format!("Command: {:?}", self);
    }
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args.command);
}
