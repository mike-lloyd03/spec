use anyhow::Result;
use serde::Serialize;

use crate::adapters::key_value::KeyValueAdapter;

pub struct IniAdapter {
    lines: Vec<String>,
}

impl IniAdapter {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn push_line(&mut self, line: &str) -> &mut Self {
        self.lines.push(line.to_string());
        self
    }

    pub fn section(&mut self, heading: &str, content: impl Serialize) -> Result<&mut Self> {
        self.lines.push(format!("[{heading}]"));

        let mut adapter = KeyValueAdapter::new("=", "#");
        adapter.parse_struct(&content)?.empty_line();
        self.lines.append(&mut adapter.lines());

        Ok(self)
    }

    pub fn build(&self) -> String {
        self.lines.join("\n")
    }
}
