use std::collections::HashMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

struct CommandContext {
    input: Option<String>,
    args: Vec<String>,
}

type CommandFn = fn(&CommandContext) -> Result<String, String>;

struct Shell {
    commands: HashMap<String, CommandFn>,
    working_dir: PathBuf,
}

impl Shell {
    fn new() -> Self {
        let mut commands = HashMap::new();
        commands.insert("echo".to_string(), Shell::echo as CommandFn);
        commands.insert("cat".to_string(), Shell::cat as CommandFn);
        commands.insert("ls".to_string(), Shell::ls as CommandFn);
        commands.insert("find".to_string(), Shell::find as CommandFn);
        commands.insert("grep".to_string(), Shell::grep as CommandFn);
        commands.insert("head".to_string(), Shell::head as CommandFn);
        commands.insert("tail".to_string(), Shell::tail as CommandFn);
        commands.insert("touch".to_string(), Shell::touch as CommandFn);
        commands.insert("cd".to_string(), Shell::cd as CommandFn);
        commands.insert("pwd".to_string(), Shell::pwd as CommandFn);
        commands.insert("exit".to_string(), Shell::exit_shell as CommandFn);
        commands.insert("help".to_string(), Shell::help as CommandFn);

        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        Shell {
            commands,
            working_dir: current_dir,
        }
    }

    fn execute(&mut self, command_line: &str) -> Result<String, String> {
        let pipeline = self.parse_pipeline(command_line);
        
        let mut previous_output: Option<String> = None;
        
        for command in pipeline {
            let parts: Vec<&str> = command.trim().split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            
            let cmd_name = parts[0];
            let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
            
            let context = CommandContext {
                input: previous_output,
                args,
            };
            
            if let Some(cmd_fn) = self.commands.get(cmd_name) {
                match cmd_fn(&context) {
                    Ok(output) => previous_output = Some(output),
                    Err(e) => return Err(format!("Error executing '{}': {}", cmd_name, e)),
                }
            } else {
                return Err(format!("Unknown command: {}", cmd_name));
            }
        }
        
        Ok(previous_output.unwrap_or_default())
    }
    
    fn parse_pipeline(&self, command_line: &str) -> Vec<String> {
        command_line.split('|').map(|s| s.trim().to_string()).collect()
    }

    // Command implementations
    fn echo(context: &CommandContext) -> Result<String, String> {
        Ok(context.args.join(" "))
    }
    
    fn cat(context: &CommandContext) -> Result<String, String> {
        if let Some(input) = &context.input {
            return Ok(input.clone());
        }
        
        if context.args.is_empty() {
            return Err("cat: missing file operand".to_string());
        }
        
        let mut result = String::new();
        for filename in &context.args {
            match fs::read_to_string(filename) {
                Ok(content) => {
                    if !result.is_empty() {
                        result.push('\n');
                    }
                    result.push_str(&content);
                }
                Err(e) => return Err(format!("cat: {}: {}", filename, e)),
            }
        }
        
        Ok(result)
    }
    
    fn ls(context: &CommandContext) -> Result<String, String> {
        let path = if context.args.is_empty() {
            Path::new(".")
        } else {
            Path::new(&context.args[0])
        };
        
        let entries = fs::read_dir(path).map_err(|e| format!("ls: {}: {}", path.display(), e))?;
        
        let mut items: Vec<String> = Vec::new();
        for entry in entries {
            if let Ok(entry) = entry {
                items.push(entry.file_name().to_string_lossy().to_string());
            }
        }
        
        items.sort();
        Ok(items.join("\n"))
    }
    
    fn find(context: &CommandContext) -> Result<String, String> {
        if context.args.len() < 1 {
            return Err("find: missing path operand".to_string());
        }
        
        let root = &context.args[0];
        let pattern = context.args.get(1).map(|s| s.as_str()).unwrap_or("*");
        
        let mut results = Vec::new();
        find_files(Path::new(root), pattern, &mut results)?;
        
        Ok(results.join("\n"))
    }
    
    fn grep(context: &CommandContext) -> Result<String, String> {
        if context.args.is_empty() {
            return Err("grep: missing pattern".to_string());
        }
        
        let pattern = &context.args[0];
        
        let content = if let Some(input) = &context.input {
            input.clone()
        } else if context.args.len() > 1 {
            let filename = &context.args[1];
            fs::read_to_string(filename).map_err(|e| format!("grep: {}: {}", filename, e))?
        } else {
            return Err("grep: no input file or pipe input".to_string());
        };
        
        let matching_lines: Vec<&str> = content
            .lines()
            .filter(|line| line.contains(pattern))
            .collect();
        
        Ok(matching_lines.join("\n"))
    }
    
    fn head(context: &CommandContext) -> Result<String, String> {
        let mut n = 10;
        let mut filename = None;
        
        let mut args_iter = context.args.iter().peekable();
        while let Some(arg) = args_iter.next() {
            if arg == "-n" {
                if let Some(num_str) = args_iter.next() {
                    n = num_str.parse::<usize>().map_err(|_| "head: invalid number of lines".to_string())?;
                } else {
                    return Err("head: option requires an argument -- 'n'".to_string());
                }
            } else {
                filename = Some(arg);
            }
        }
        
        let content = if let Some(input) = &context.input {
            input.clone()
        } else if let Some(file) = filename {
            fs::read_to_string(file).map_err(|e| format!("head: {}: {}", file, e))?
        } else {
            return Err("head: no input file or pipe input".to_string());
        };
        
        let lines: Vec<&str> = content.lines().take(n).collect();
        Ok(lines.join("\n"))
    }
    
    fn tail(context: &CommandContext) -> Result<String, String> {
        let mut n = 10;
        let mut filename = None;
        
        let mut args_iter = context.args.iter().peekable();
        while let Some(arg) = args_iter.next() {
            if arg == "-n" {
                if let Some(num_str) = args_iter.next() {
                    n = num_str.parse::<usize>().map_err(|_| "tail: invalid number of lines".to_string())?;
                } else {
                    return Err("tail: option requires an argument -- 'n'".to_string());
                }
            } else {
                filename = Some(arg);
            }
        }
        
        let content = if let Some(input) = &context.input {
            input.clone()
        } else if let Some(file) = filename {
            fs::read_to_string(file).map_err(|e| format!("tail: {}: {}", file, e))?
        } else {
            return Err("tail: no input file or pipe input".to_string());
        };
        
        let lines: Vec<&str> = content.lines().collect();
        let start = if lines.len() > n { lines.len() - n } else { 0 };
        
        Ok(lines[start..].join("\n"))
    }
    
    fn touch(context: &CommandContext) -> Result<String, String> {
        if context.args.is_empty() {
            return Err("touch: missing file operand".to_string());
        }
        
        for filename in &context.args {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(filename)
                .map_err(|e| format!("touch: {}: {}", filename, e))?;
            drop(file);
        }
        
        Ok(String::new())
    }
    
    fn cd(context: &CommandContext) -> Result<String, String> {
        let path = if context.args.is_empty() {
            PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".to_string()))
        } else {
            PathBuf::from(&context.args[0])
        };
        
        env::set_current_dir(&path).map_err(|e| format!("cd: {}: {}", path.display(), e))?;
        Ok(String::new())
    }
    
    fn pwd(_context: &CommandContext) -> Result<String, String> {
        let path = env::current_dir().map_err(|e| format!("pwd: {}", e))?;
        Ok(path.to_string_lossy().to_string())
    }
    
    fn exit_shell(_context: &CommandContext) -> Result<String, String> {
        println!("Exiting shell");
        exit(0);
    }
    
    fn help(_context: &CommandContext) -> Result<String, String> {
        Ok("Available commands: echo, cat, ls, find, grep, head, tail, touch, cd, pwd, exit, help\n\
            Use | for piping commands together.".to_string())
    }
}

fn find_files(dir: &Path, pattern: &str, results: &mut Vec<String>) -> Result<(), String> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).map_err(|e| format!("find: {}: {}", dir.display(), e))? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    find_files(&path, pattern, results)?;
                } else {
                    let filename = path.file_name().unwrap_or_default().to_string_lossy();
                    if pattern == "*" || filename.contains(pattern) {
                        results.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    Ok(())
}

fn main() {
    let mut shell = Shell::new();
    
    println!("Welcome to Rust Shell! Type 'help' for available commands or 'exit' to quit.");
    
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("Error reading input");
            continue;
        }
        
        input = input.trim().to_string();
        if input.is_empty() {
            continue;
        }
        
        match shell.execute(&input) {
            Ok(output) => {
                if !output.is_empty() {
                    println!("{}", output);
                }
            }
            Err(e) => eprintln!("{}", e),
        }
    }
}