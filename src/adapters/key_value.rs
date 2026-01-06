use color_eyre::Result;
use serde::Serialize;
use toml::Value;

pub enum BoolStyle {
    TrueFalse,
    YesNo,
    OnOff,
}

pub struct KeyValueAdapter {
    separator: String,
    comment_char: String,
    bool_style: BoolStyle,
    lines: Vec<String>,
}

impl KeyValueAdapter {
    pub fn new(separator: &str, comment_char: &str) -> Self {
        Self {
            separator: separator.to_string(),
            comment_char: comment_char.to_string(),
            bool_style: BoolStyle::TrueFalse,
            lines: Vec::new(),
        }
    }

    pub fn bool_style(mut self, style: BoolStyle) -> Self {
        self.bool_style = style;
        self
    }

    pub fn comment(&mut self, text: &str) -> &mut Self {
        self.lines.push(format!("{} {}", self.comment_char, text));
        self
    }

    pub fn set<V: ToString>(&mut self, key: &str, value: V) -> &mut Self {
        self.lines
            .push(format!("{}{}{}", key, self.separator, value.to_string()));
        self
    }

    pub fn parse_struct<T: Serialize>(&mut self, data: &T) -> Result<&mut Self> {
        let val = Value::try_from(data)?;

        if let Value::Table(map) = val {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();

            for key in keys {
                let toml_val = &map[key];
                self.process_value(key, toml_val);
            }
        }
        Ok(self)
    }

    fn process_value(&mut self, key: &str, val: &Value) {
        match val {
            Value::String(s) => {
                self.set(key, s);
            }
            Value::Integer(i) => {
                self.set(key, i);
            }
            Value::Float(f) => {
                self.set(key, f);
            }
            Value::Boolean(b) => {
                let s = self.format_bool(*b);
                self.set(key, s);
            }
            // HANDLE ARRAYS (Vec<String> in struct)
            Value::Array(arr) => {
                for item in arr {
                    // Recursively call to print "Key Value" for every item in array
                    self.process_value(key, item);
                }
            }
            _ => {} // Skip tables/inline-tables for flat KV files
        }
    }

    fn format_bool(&self, val: bool) -> String {
        match self.bool_style {
            BoolStyle::TrueFalse => val.to_string(),
            BoolStyle::YesNo => if val { "yes" } else { "no" }.to_string(),
            BoolStyle::OnOff => if val { "on" } else { "off" }.to_string(),
        }
    }

    pub fn build(&self) -> String {
        self.lines.join("\n")
    }
}
