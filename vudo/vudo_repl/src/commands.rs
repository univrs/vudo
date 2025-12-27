use crate::environment::ReplEnvironment;
use crate::printer::Printer;
use std::fs;
use std::io::Write;

#[derive(Debug)]
pub enum ReplCommand {
    Help,
    Quit,
    Clear,
    Reset,
    Load(String),
    Save(String),
    History,
    Type(String),
    Ast(String),
    Mlir(String),
    Wasm(String),
    Env,
    Set(String, String),
    Get(String),
    Eval(String),
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl ReplCommand {
    pub fn parse(line: &str) -> Result<Self, CommandError> {
        let line = line.trim();

        if line.is_empty() {
            return Ok(ReplCommand::Eval(String::new()));
        }

        if !line.starts_with(':') {
            return Ok(ReplCommand::Eval(line.to_string()));
        }

        let parts: Vec<&str> = line[1..].split_whitespace().collect();
        if parts.is_empty() {
            return Err(CommandError::InvalidArguments("Empty command".to_string()));
        }

        match parts[0] {
            "help" | "h" => Ok(ReplCommand::Help),
            "quit" | "q" => Ok(ReplCommand::Quit),
            "clear" | "c" => Ok(ReplCommand::Clear),
            "reset" => Ok(ReplCommand::Reset),
            "load" => {
                if parts.len() < 2 {
                    Err(CommandError::InvalidArguments(
                        "Usage: :load <file>".to_string(),
                    ))
                } else {
                    Ok(ReplCommand::Load(parts[1].to_string()))
                }
            }
            "save" => {
                if parts.len() < 2 {
                    Err(CommandError::InvalidArguments(
                        "Usage: :save <file>".to_string(),
                    ))
                } else {
                    Ok(ReplCommand::Save(parts[1].to_string()))
                }
            }
            "history" => Ok(ReplCommand::History),
            "type" => {
                if parts.len() < 2 {
                    Err(CommandError::InvalidArguments(
                        "Usage: :type <expr>".to_string(),
                    ))
                } else {
                    Ok(ReplCommand::Type(parts[1..].join(" ")))
                }
            }
            "ast" => {
                if parts.len() < 2 {
                    Err(CommandError::InvalidArguments(
                        "Usage: :ast <expr>".to_string(),
                    ))
                } else {
                    Ok(ReplCommand::Ast(parts[1..].join(" ")))
                }
            }
            "mlir" => {
                if parts.len() < 2 {
                    Err(CommandError::InvalidArguments(
                        "Usage: :mlir <expr>".to_string(),
                    ))
                } else {
                    Ok(ReplCommand::Mlir(parts[1..].join(" ")))
                }
            }
            "wasm" => {
                if parts.len() < 2 {
                    Err(CommandError::InvalidArguments(
                        "Usage: :wasm <expr>".to_string(),
                    ))
                } else {
                    Ok(ReplCommand::Wasm(parts[1..].join(" ")))
                }
            }
            "env" => Ok(ReplCommand::Env),
            "set" => {
                if parts.len() < 3 {
                    Err(CommandError::InvalidArguments(
                        "Usage: :set <option> <value>".to_string(),
                    ))
                } else {
                    Ok(ReplCommand::Set(parts[1].to_string(), parts[2..].join(" ")))
                }
            }
            "get" => {
                if parts.len() < 2 {
                    Err(CommandError::InvalidArguments(
                        "Usage: :get <option>".to_string(),
                    ))
                } else {
                    Ok(ReplCommand::Get(parts[1].to_string()))
                }
            }
            cmd => Err(CommandError::UnknownCommand(cmd.to_string())),
        }
    }

    pub fn execute(
        &self,
        env: &mut ReplEnvironment,
        history: &[String],
    ) -> Result<bool, CommandError> {
        match self {
            ReplCommand::Help => {
                Printer::print_help();
                Ok(false)
            }
            ReplCommand::Quit => {
                Printer::print_farewell();
                Ok(true)
            }
            ReplCommand::Clear => {
                print!("\x1B[2J\x1B[1;1H");
                std::io::stdout().flush()?;
                Ok(false)
            }
            ReplCommand::Reset => {
                env.reset();
                Printer::print_success("Environment reset");
                Ok(false)
            }
            ReplCommand::Load(file) => {
                match fs::read_to_string(file) {
                    Ok(contents) => {
                        Printer::print_success(&format!(
                            "Loaded {} bytes from {}",
                            contents.len(),
                            file
                        ));
                        // TODO: Actually parse and execute the DOL file
                        Printer::print_info(
                            "File loaded but not yet executed (parser not implemented)",
                        );
                    }
                    Err(e) => {
                        Printer::print_error(&format!("Failed to load {}: {}", file, e));
                    }
                }
                Ok(false)
            }
            ReplCommand::Save(file) => {
                let session_data = history.join("\n");
                match fs::write(file, session_data) {
                    Ok(_) => {
                        Printer::print_success(&format!("Session saved to {}", file));
                    }
                    Err(e) => {
                        Printer::print_error(&format!("Failed to save to {}: {}", file, e));
                    }
                }
                Ok(false)
            }
            ReplCommand::History => {
                println!("Command History:");
                for (i, cmd) in history.iter().enumerate() {
                    println!("{:4}: {}", i + 1, cmd);
                }
                Ok(false)
            }
            ReplCommand::Type(expr) => {
                Printer::print_info(&format!("Type analysis for: {}", expr));
                Printer::print_warning("Type inference not yet implemented");
                println!("  => Type: <unknown>");
                Ok(false)
            }
            ReplCommand::Ast(expr) => {
                Printer::print_info(&format!("AST for: {}", expr));
                Printer::print_warning("AST generation not yet implemented");
                println!("  => AST: <not available>");
                Ok(false)
            }
            ReplCommand::Mlir(expr) => {
                Printer::print_info(&format!("MLIR for: {}", expr));
                Printer::print_warning("MLIR generation not yet implemented");
                println!("  => MLIR: <not available>");
                Ok(false)
            }
            ReplCommand::Wasm(expr) => {
                Printer::print_info(&format!("WASM for: {}", expr));
                Printer::print_warning("WASM generation not yet implemented");
                println!("  => WASM: <not available>");
                Ok(false)
            }
            ReplCommand::Env => {
                println!("Environment Symbols:");
                if env.symbols().is_empty() {
                    println!("  (no symbols defined)");
                } else {
                    for (name, value) in env.symbols() {
                        println!("  {} = {}", name, value);
                    }
                }
                println!();
                println!("Options:");
                for (key, value) in env.options() {
                    println!("  {} = {}", key, value);
                }
                Ok(false)
            }
            ReplCommand::Set(key, value) => {
                env.set_option(key.clone(), value.clone());
                Printer::print_success(&format!("Set {} = {}", key, value));
                Ok(false)
            }
            ReplCommand::Get(key) => {
                match env.get_option(key) {
                    Some(value) => {
                        println!("{} = {}", key, value);
                    }
                    None => {
                        Printer::print_warning(&format!("Option '{}' not found", key));
                    }
                }
                Ok(false)
            }
            ReplCommand::Eval(expr) => {
                if expr.trim().is_empty() {
                    return Ok(false);
                }

                // TODO: Implement actual DOL expression evaluation
                Printer::print_info(&format!("Evaluating: {}", expr));
                Printer::print_warning("DOL parser and evaluator not yet implemented");
                println!("  => Result: <evaluation not available>");

                Ok(false)
            }
        }
    }
}
