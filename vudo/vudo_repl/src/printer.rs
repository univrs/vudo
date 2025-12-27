use colored::*;

pub struct Printer;

impl Printer {
    pub fn print_banner() {
        println!(
            "{}",
            "╔══════════════════════════════════════════════════════════════════╗"
                .cyan()
                .bold()
        );
        println!(
            "{}",
            "║  VUDO DOL REPL v0.1.0                                            ║"
                .cyan()
                .bold()
        );
        println!(
            "{}",
            "║  The system that knows what it is, becomes what it knows.        ║"
                .cyan()
                .bold()
        );
        println!(
            "{}",
            "║                                                                  ║"
                .cyan()
                .bold()
        );
        println!(
            "{}",
            "║  Type :help for commands, :quit to exit                          ║"
                .cyan()
                .bold()
        );
        println!(
            "{}",
            "╚══════════════════════════════════════════════════════════════════╝"
                .cyan()
                .bold()
        );
        println!();
    }

    pub fn print_farewell() {
        println!(
            "{}",
            "Goodbye! May your Spirits thrive in Bondieu. 🍄"
                .green()
                .bold()
        );
    }

    pub fn print_error(msg: &str) {
        println!("{} {}", "Error:".red().bold(), msg);
    }

    pub fn print_warning(msg: &str) {
        println!("{} {}", "Warning:".yellow().bold(), msg);
    }

    pub fn print_info(msg: &str) {
        println!("{} {}", "Info:".blue().bold(), msg);
    }

    pub fn print_success(msg: &str) {
        println!("{} {}", "Success:".green().bold(), msg);
    }

    pub fn print_help() {
        println!("{}", "Available Commands:".cyan().bold());
        println!();
        println!("  {}  Show this help message", ":help, :h".green());
        println!("  {}  Exit the REPL", ":quit, :q".green());
        println!("  {}  Clear the screen", ":clear, :c".green());
        println!("  {}  Reset the environment", ":reset".green());
        println!("  {}  Load a .dol file", ":load <file>".green());
        println!("  {}  Save current session to file", ":save <file>".green());
        println!("  {}  Show command history", ":history".green());
        println!("  {}  Show type of expression", ":type <expr>".green());
        println!("  {}  Show AST of expression", ":ast <expr>".green());
        println!("  {}  Show MLIR of expression", ":mlir <expr>".green());
        println!("  {}  Show WASM of expression", ":wasm <expr>".green());
        println!("  {}  Show defined symbols", ":env".green());
        println!("  {}  Set a REPL option", ":set <option> <value>".green());
        println!("  {}  Get a REPL option value", ":get <option>".green());
        println!();
        println!("{}", "DOL Expressions:".cyan().bold());
        println!("  Enter any valid DOL expression to evaluate it");
        println!();
    }

    pub fn print_result(result: &str) {
        println!("{}", result);
    }
}
