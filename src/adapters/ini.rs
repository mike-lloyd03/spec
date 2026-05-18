use anyhow::Result;
use serde::Serialize;

use crate::adapters::key_value::KeyValueAdapter;

pub struct IniAdapter {
    lines: Vec<String>,
    separator: String,
}

impl IniAdapter {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            separator: "=".to_string(),
        }
    }

    pub fn push_line(&mut self, line: &str) -> &mut Self {
        self.lines.push(line.to_string());
        self
    }

    pub fn spaces_around_equals(&mut self, enable: bool) -> &mut Self {
        match enable {
            true => self.separator = " = ".to_string(),
            false => self.separator = "=".to_string(),
        }
        self
    }

    pub fn section(&mut self, heading: &str, content: impl Serialize) -> Result<&mut Self> {
        let mut adapter = KeyValueAdapter::new(&self.separator, "#").sequence_separator(",");
        if adapter.parse_struct(&content).is_ok() {
            self.lines.push(format!("[{heading}]"));
            adapter.empty_line();
            self.lines.append(&mut adapter.lines());
        }

        Ok(self)
    }

    pub fn build(&self) -> String {
        self.lines.join("\n")
    }
}
