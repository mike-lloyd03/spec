use anyhow::{Result, bail};
use serde::Serialize;
use toml::{Table, Value};

pub enum BoolStyle {
    TrueFalse,
    YesNo,
    OneZero,
}

pub struct KeyValueAdapter {
    separator: String,
    comment_char: String,
    bool_style: BoolStyle,
    lines: Vec<String>,
    indent_level: usize,
    indent_width: usize,
    sequence_separator: String,
}

impl KeyValueAdapter {
    pub fn new(separator: &str, comment_char: &str) -> Self {
        Self {
            separator: separator.to_string(),
            comment_char: comment_char.to_string(),
            bool_style: BoolStyle::TrueFalse,
            lines: Vec::new(),
            indent_level: 0,
            indent_width: 4,
            sequence_separator: "multi".to_string(),
        }
    }

    pub fn bool_style(mut self, style: BoolStyle) -> Self {
        self.bool_style = style;
        self
    }

    pub fn sequence_separator(mut self, separator: &str) -> Self {
        self.sequence_separator = separator.to_string();
        self
    }

    pub fn comment(&mut self, text: &str) -> &mut Self {
        self.lines.push(format!("{} {}", self.comment_char, text));
        self
    }

    pub fn set<V: ToString>(&mut self, key: &str, value: V) -> &mut Self {
        let indent = " ".repeat(self.indent_level * self.indent_width);

        self.lines.push(format!(
            "{}{}{}{}",
            indent,
            key,
            self.separator,
            value.to_string()
        ));
        self
    }

    pub fn parse_struct<T: Serialize>(&mut self, data: &T) -> Result<&mut Self> {
        let table = Table::try_from(data)?;
        if table.is_empty() {
            bail!("Configuration table is empty")
        }

        for (k, v) in table {
            self.process_value(&k, &v);
        }

        Ok(self)
    }

    fn process_value(&mut self, key: &str, val: &Value) {
        if let Value::Array(arr) = val {
            if self.sequence_separator == "multi" {
                for item in arr {
                    self.process_value(key, item);
                }
            } else {
                self.set(key, self.stringify(val));
            }
        } else {
            self.set(key, self.stringify(val));
        }
    }

    fn stringify(&self, val: &Value) -> String {
        match val {
            Value::String(s) => s.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Boolean(b) => match self.bool_style {
                BoolStyle::TrueFalse => b.to_string(),
                BoolStyle::YesNo => if *b { "yes" } else { "no" }.to_string(),
                BoolStyle::OneZero => if *b { "1" } else { "0" }.to_string(),
            },
            Value::Datetime(d) => d.to_string(),
            Value::Array(arr) => {
                let s: Vec<String> = arr.iter().map(|v| self.stringify(v)).collect();
                s.join(&self.sequence_separator)
            }
            Value::Table(table) => {
                let mut values = Vec::new();
                for (k, v) in table {
                    values.push(format!("'{}={}'", k, self.stringify(v)))
                }
                values.join(" ")
            }
        }
    }

    pub fn indent(&mut self) -> &mut Self {
        self.indent_level += 1;
        self
    }

    pub fn outdent(&mut self) -> &mut Self {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        } else {
            self.indent_level = 0;
        }
        self
    }

    pub fn empty_line(&mut self) -> &mut Self {
        self.lines.push("".to_string());
        self
    }

    pub fn build(&self) -> String {
        self.lines.join("\n")
    }

    pub fn lines(&self) -> Vec<String> {
        self.lines.clone()
    }

    pub fn push_line(&mut self, line: &str) -> &mut Self {
        self.lines.push(line.to_string());
        self
    }
}
