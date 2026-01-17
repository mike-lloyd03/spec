use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

fn parse_bool_de<'de, D, T>(deserializer: D) -> std::result::Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + From<bool>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => match s.to_lowercase().as_str() {
            "true" => Ok(true.into()),
            "false" => Ok(false.into()),
            _ => Err(de::Error::custom("invalid boolean value")),
        },
        None => Ok(T::default()),
    }
}

pub fn serialize_bool<S>(t: &Option<bool>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(b) = t {
        match b {
            true => s.serialize_str("true"),
            false => s.serialize_str("false"),
        }
    } else {
        s.serialize_none()
    }
}
