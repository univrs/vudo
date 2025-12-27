use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ReplEnvironment {
    symbols: HashMap<String, String>,
    options: HashMap<String, String>,
}

impl ReplEnvironment {
    pub fn new() -> Self {
        let mut env = Self {
            symbols: HashMap::new(),
            options: HashMap::new(),
        };

        // Set default options
        env.options
            .insert("verbose".to_string(), "false".to_string());
        env.options
            .insert("show_types".to_string(), "true".to_string());
        env.options.insert("color".to_string(), "true".to_string());

        env
    }

    pub fn define(&mut self, name: String, value: String) {
        self.symbols.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&String> {
        self.symbols.get(name)
    }

    pub fn symbols(&self) -> &HashMap<String, String> {
        &self.symbols
    }

    pub fn reset(&mut self) {
        self.symbols.clear();
    }

    pub fn set_option(&mut self, key: String, value: String) {
        self.options.insert(key, value);
    }

    pub fn get_option(&self, key: &str) -> Option<&String> {
        self.options.get(key)
    }

    pub fn options(&self) -> &HashMap<String, String> {
        &self.options
    }
}

impl Default for ReplEnvironment {
    fn default() -> Self {
        Self::new()
    }
}
