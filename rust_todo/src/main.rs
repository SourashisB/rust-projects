use clap::{Parser, Subcommand};
use serde::{Serialize, Deserialize};
use std::fs;
use std::io::{self, Write};

const FILE: &str = "todo.json";

#[derive(Serialize, Deserialize, Debug)]
struct Todo {
    tasks: Vec<String>,
}

impl Todo {
    fn new() -> Self {
        Self { tasks: vec![] }
    }
    fn add(&mut self, task: String) {
        self.tasks.push(task);
    }
    fn list(&self) {
        if self.tasks.is_empty() {
            println!("No tasks yet!");
        } else {
            for (i, task) in self.tasks.iter().enumerate() {
                println!("{}: {}", i + 1, task);
            }
        }
    }
    fn remove(&mut self, index: usize) {
        if index == 0 || index > self.tasks.len() {
            println!("Invalid task number.");
        } else {
            self.tasks.remove(index - 1);
            println!("Task removed.");
        }
    }
    fn save(&self) -> io::Result<()> {
        let data = serde_json::to_string(&self).unwrap();
        fs::write(FILE, data)
    }
    fn load() -> Self {
        fs::read_to_string(FILE)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_else(Todo::new)
    }
}

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task
    Add { task: Vec<String> },
    /// List all tasks
    List,
    /// Remove a task by its number
    Remove { number: usize },
}

fn main() {
    let cli = Cli::parse();
    let mut todo = Todo::load();

    match cli.command {
        Some(Commands::Add { task }) => {
            let task_str = task.join(" ");
            todo.add(task_str);
            todo.save().expect("Could not save todo list");
            println!("Task added.");
        }
        Some(Commands::List) => {
            todo.list();
        }
        Some(Commands::Remove { number }) => {
            todo.remove(number);
            todo.save().expect("Could not save todo list");
        }
        None => {
            println!("No command provided. Use --help for more info.");
        }
    }
}