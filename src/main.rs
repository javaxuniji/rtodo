use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(name = "rtodo", version, about = "A simple todo CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Add a new todo item
    Add {
        /// The text of the todo item
        text: Vec<String>,
    },
    /// List todo items
    List,
    /// Mark a todo item as done
    Done {
        /// The id of the todo item to mark as done
        id: u32,
    },
    /// Remove a todo item
    Remove {
        /// The id of the todo item to remove
        id: u32,
    },
    /// Clear all todo items
    Clear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TodoItem {
    id: u32,
    text: String,
    done: bool,
}

impl fmt::Display for TodoItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.done { "x" } else { " " };
        write!(f, "[{}] {}: {}", status, self.id, self.text)
    }
}

fn main() {
    let cli = Cli::parse();
    let store_path = todo_store_path();

    if let Err(err) = handle_command(cli.command, &store_path) {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn handle_command(command: Commands, store_path: &Path) -> Result<(), String> {
    let mut todos = load_todos(store_path)?;

    match command {
        Commands::Add { text } => {
            let description = text.join(" ").trim().to_string();
            if description.is_empty() {
                return Err("todo text cannot be empty".to_string());
            }

            let next_id = todos.iter().map(|item| item.id).max().unwrap_or(0) + 1;
            let item = TodoItem {
                id: next_id,
                text: description,
                done: false,
            };
            todos.push(item);
            save_todos(store_path, &todos)?;
            println!("Added todo #{next_id}");
        }
        Commands::List => {
            if todos.is_empty() {
                println!("No todos yet.");
            } else {
                for todo in todos {
                    println!("{todo}");
                }
            }
        }
        Commands::Done { id } => {
            let todo = todos.iter_mut().find(|item| item.id == id);
            match todo {
                Some(item) => {
                    item.done = true;
                    save_todos(store_path, &todos)?;
                    println!("Marked todo #{id} as done");
                }
                None => return Err(format!("todo #{id} not found")),
            }
        }
        Commands::Remove { id } => {
            let initial_len = todos.len();
            todos.retain(|item| item.id != id);
            if todos.len() == initial_len {
                return Err(format!("todo #{id} not found"));
            }
            save_todos(store_path, &todos)?;
            println!("Removed todo #{id}");
        }
        Commands::Clear => {
            todos.clear();
            save_todos(store_path, &todos)?;
            println!("Cleared all todos");
        }
    }

    Ok(())
}

fn todo_store_path() -> PathBuf {
    if let Ok(path) = env::var("RTODO_FILE") {
        return PathBuf::from(path);
    }

    env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".rtodo.json")
}

fn load_todos(store_path: &Path) -> Result<Vec<TodoItem>, String> {
    if !store_path.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(store_path)
        .map_err(|err| format!("failed to read {:?}: {err}", store_path))?;
    if contents.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(&contents)
        .map_err(|err| format!("failed to parse {:?}: {err}", store_path))
}

fn save_todos(store_path: &Path, todos: &[TodoItem]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(todos)
        .map_err(|err| format!("failed to serialize todos: {err}"))?;
    fs::write(store_path, json)
        .map_err(|err| format!("failed to write {:?}: {err}", store_path))
}
