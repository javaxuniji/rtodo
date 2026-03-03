use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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
        /// Add a note to the todo item
        #[arg(long)]
        note: Option<String>,
        #[arg(short = 'p', long, default_value = "m", value_parser = parse_priority)]
        priority: Priority,
        #[arg(long = "at", alias = "time", value_parser = parse_added_at)]
        at: Option<i64>,
    },
    /// List todo items
    List {
        /// Show all todo items (including completed ones)
        #[arg(long)]
        all: bool,
    },
    /// Show the todo data file path
    Path,
    /// Mark a todo item as done
    Done {
        /// The id of the todo item to mark as done
        id: u32,
        /// Add a note when marking as done
        #[arg(long)]
        note: Option<String>,
    },
    /// Remove a todo item
    Remove {
        /// The id of the todo item to remove
        id: u32,
        /// Skip interactive confirmation prompts
        #[arg(long)]
        yes: bool,
    },
    /// Clear all todo items
    Clear {
        /// Skip interactive confirmation prompts
        #[arg(long)]
        yes: bool,
    },
    /// Show version information
    Version,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TodoItem {
    id: u32,
    text: String,
    done: bool,
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    priority: Priority,
    #[serde(default)]
    added_at: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
enum Priority {
    Low,
    Medium,
    High,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Medium
    }
}

impl Priority {
    fn short(self) -> char {
        match self {
            Priority::Low => 'l',
            Priority::Medium => 'm',
            Priority::High => 'h',
        }
    }
}

impl fmt::Display for TodoItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.done { "x" } else { " " };
        write!(f, "[{}] {}: {}", status, self.id, self.text)?;
        write!(f, " [{}]", self.priority.short())?;
        if self.added_at != 0 {
            write!(f, " @{}", self.added_at)?;
        }
        if let Some(note) = &self.note {
            write!(f, " ({})", note)?;
        }
        Ok(())
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
        Commands::Add {
            text,
            note,
            priority,
            at,
        } => {
            let description = text.join(" ").trim().to_string();
            if description.is_empty() {
                return Err("todo text cannot be empty".to_string());
            }

            let next_id = todos.iter().map(|item| item.id).max().unwrap_or(0) + 1;
            let added_at = at.unwrap_or_else(now_unix_seconds);
            let item = TodoItem {
                id: next_id,
                text: description,
                done: false,
                note,
                priority,
                added_at,
            };
            todos.push(item);
            save_todos(store_path, &todos)?;
            println!("Added todo #{next_id}");
        }
        Commands::List { all } => {
            if todos.is_empty() {
                println!("No todos yet.");
            } else {
                todos.sort_by_key(|todo| (todo.added_at, todo.id));
                for todo in todos {
                    if all || !todo.done {
                        println!("{todo}");
                    }
                }
            }
        }
        Commands::Path => {
            println!("{}", store_path.display());
        }
        Commands::Done { id, note } => {
            let todo = todos.iter_mut().find(|item| item.id == id);
            match todo {
                Some(item) => {
                    item.done = true;
                    if let Some(n) = note {
                        item.note = Some(n);
                    }
                    save_todos(store_path, &todos)?;
                    println!("Marked todo #{id} as done");
                }
                None => return Err(format!("todo #{id} not found")),
            }
        }
        Commands::Remove { id, yes } => {
            if !yes {
                confirm_deletion(&format!("todo #{id}"))?;
            }

            let initial_len = todos.len();
            todos.retain(|item| item.id != id);
            if todos.len() == initial_len {
                return Err(format!("todo #{id} not found"));
            }
            save_todos(store_path, &todos)?;
            println!("Removed todo #{id}");
        }
        Commands::Clear { yes } => {
            if !yes {
                confirm_deletion("all todos")?;
            }

            todos.clear();
            save_todos(store_path, &todos)?;
            println!("Cleared all todos");
        }
        Commands::Version => {
            println!("rtodo version {}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}

fn todo_store_path() -> PathBuf {
    if let Ok(path) = env::var("RTODO_FILE") {
        return PathBuf::from(path);
    }

    let home = user_home_dir()
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    home.join(".rtodo").join("todos.json")
}

fn user_home_dir() -> Option<PathBuf> {
    if cfg!(windows) {
        if let Some(user_profile) = env::var_os("USERPROFILE") {
            return Some(PathBuf::from(user_profile));
        }

        let home_drive = env::var_os("HOMEDRIVE")?;
        let home_path = env::var_os("HOMEPATH")?;
        let mut path = PathBuf::from(home_drive);
        path.push(home_path);
        return Some(path);
    }

    if let Some(home) = env::var_os("HOME") {
        return Some(PathBuf::from(home));
    }

    if let Some(user_profile) = env::var_os("USERPROFILE") {
        return Some(PathBuf::from(user_profile));
    }

    let home_drive = env::var_os("HOMEDRIVE")?;
    let home_path = env::var_os("HOMEPATH")?;

    let mut path = PathBuf::from(home_drive);
    path.push(home_path);
    Some(path)
}

fn confirm_deletion(target: &str) -> Result<(), String> {
    if !ask_confirm(&format!("Delete {target}? (y/N): "))? {
        return Err("aborted: deletion canceled".to_string());
    }

    if !ask_confirm("Please confirm again, type y to proceed (y/N): ")? {
        return Err("aborted: deletion canceled".to_string());
    }

    Ok(())
}

fn ask_confirm(prompt: &str) -> Result<bool, String> {
    print!("{prompt}");
    io::stdout()
        .flush()
        .map_err(|err| format!("failed to flush stdout: {err}"))?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|err| format!("failed to read confirmation: {err}"))?;

    Ok(matches!(
        input.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
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
    if let Some(parent) = store_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {:?}: {err}", parent))?;
    }

    let json = serde_json::to_string_pretty(todos)
        .map_err(|err| format!("failed to serialize todos: {err}"))?;
    fs::write(store_path, json).map_err(|err| format!("failed to write {:?}: {err}", store_path))
}

fn parse_priority(value: &str) -> Result<Priority, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "l" | "low" => Ok(Priority::Low),
        "m" | "med" | "medium" => Ok(Priority::Medium),
        "h" | "hi" | "high" => Ok(Priority::High),
        _ => Err("invalid priority: use low|l, medium|m, high|h".to_string()),
    }
}

fn parse_added_at(value: &str) -> Result<i64, String> {
    let v = value.trim().to_ascii_lowercase();
    if v == "now" {
        return Ok(now_unix_seconds());
    }
    let raw: i64 = v
        .parse()
        .map_err(|_| "invalid added time: use unix seconds, unix milliseconds, or \"now\"".to_string())?;
    if raw >= 1_000_000_000_000 {
        Ok(raw / 1000)
    } else {
        Ok(raw)
    }
}

fn now_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
