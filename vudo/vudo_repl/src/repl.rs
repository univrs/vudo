use crate::commands::{CommandError, ReplCommand};
use crate::environment::ReplEnvironment;
use crate::printer::Printer;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RustylineResult};

pub struct ReplConfig {
    pub show_banner: bool,
    pub load_file: Option<String>,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            show_banner: true,
            load_file: None,
        }
    }
}

pub struct Repl {
    editor: DefaultEditor,
    environment: ReplEnvironment,
    history: Vec<String>,
    config: ReplConfig,
}

impl Repl {
    pub fn new(config: ReplConfig) -> RustylineResult<Self> {
        let mut editor = DefaultEditor::new()?;

        // Set up history file path
        if let Some(home) = dirs::home_dir() {
            let history_path = home.join(".vudo").join("history");
            if let Some(parent) = history_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = editor.load_history(&history_path);
        }

        Ok(Self {
            editor,
            environment: ReplEnvironment::new(),
            history: Vec::new(),
            config,
        })
    }

    pub fn run(&mut self) -> RustylineResult<()> {
        if self.config.show_banner {
            Printer::print_banner();
        }

        // Load file if specified
        if let Some(ref file) = self.config.load_file {
            let cmd = ReplCommand::Load(file.clone());
            if let Err(e) = cmd.execute(&mut self.environment, &self.history) {
                Printer::print_error(&format!("Failed to load file: {}", e));
            }
        }

        loop {
            let prompt = "DOL> ";

            match self.editor.readline(prompt) {
                Ok(line) => {
                    let line = line.trim();

                    if line.is_empty() {
                        continue;
                    }

                    // Add to history
                    self.editor.add_history_entry(line)?;
                    self.history.push(line.to_string());

                    // Parse and execute command
                    match ReplCommand::parse(line) {
                        Ok(cmd) => match cmd.execute(&mut self.environment, &self.history) {
                            Ok(should_quit) => {
                                if should_quit {
                                    break;
                                }
                            }
                            Err(e) => {
                                Printer::print_error(&format!("Command execution failed: {}", e));
                            }
                        },
                        Err(CommandError::UnknownCommand(cmd)) => {
                            Printer::print_error(&format!("Unknown command: :{}", cmd));
                            println!("Type :help for available commands");
                        }
                        Err(CommandError::InvalidArguments(msg)) => {
                            Printer::print_error(&msg);
                        }
                        Err(e) => {
                            Printer::print_error(&format!("Parse error: {}", e));
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    Printer::print_info("Use :quit or :q to exit");
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                    Printer::print_farewell();
                    break;
                }
                Err(err) => {
                    Printer::print_error(&format!("Readline error: {}", err));
                    break;
                }
            }
        }

        // Save history
        if let Some(home) = dirs::home_dir() {
            let history_path = home.join(".vudo").join("history");
            let _ = self.editor.save_history(&history_path);
        }

        Ok(())
    }
}

// Helper function to get home directory
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }
}
