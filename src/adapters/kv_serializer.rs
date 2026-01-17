use std::{fmt::Display, result::Result};

use serde::{Serialize, ser, ser::Impossible};

#[derive(Debug, thiserror::Error)]
pub enum Serror {
    #[error("failed to parse value")]
    Msg(String),
}

impl ser::Error for Serror {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Msg(msg.to_string())
    }
}

pub struct KeyValueSerializer {
    output: String,
    separator: String,
    seq_separator: String,
    comment_char: String,
}

impl KeyValueSerializer {
    pub fn new(separator: &str) -> Self {
        Self {
            separator: separator.to_string(),
            comment_char: "# ".to_string(),
            seq_separator: ", ".to_string(),
            output: String::new(),
        }
    }

    pub fn set_comment_char(&mut self, comment_char: &str) -> &mut Self {
        self.comment_char = comment_char.to_string();
        self
    }

    pub fn set_seq_separator(&mut self, separator: &str) -> &mut Self {
        self.seq_separator = separator.to_string();
        self
    }

    pub fn to_string<T: Serialize>(&mut self, input: T) -> Result<String, Serror> {
        self.output = String::new();
        input.serialize(&mut *self)?;
        Ok(self.output.clone())
    }

    pub fn comment(&mut self, comment: &str) -> &mut Self {
        self.output += comment;
        self
    }
}

impl ser::Serializer for &mut KeyValueSerializer {
    type Ok = ();

    type Error = Serror;

    type SerializeSeq = Self;

    type SerializeTuple = Impossible<Self::Ok, Self::Error>;

    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;

    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;

    type SerializeMap = Impossible<Self::Ok, Self::Error>;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.output += if v { "true" } else { "false" };
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.output += v;
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Serror::Msg("Not implemented".into()))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Serror::Msg("Not implemented".into()))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(self)
    }
}

impl ser::SerializeSeq for &mut KeyValueSerializer {
    type Ok = ();
    type Error = Serror;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        self.output += &self.seq_separator;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl ser::SerializeStruct for &mut KeyValueSerializer {
    type Ok = ();
    type Error = Serror;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Serror>
    where
        T: ?Sized + Serialize,
    {
        struct_to_key_value(self, key, value)
    }

    fn end(self) -> Result<(), Serror> {
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut KeyValueSerializer {
    type Ok = ();
    type Error = Serror;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Serror>
    where
        T: ?Sized + Serialize,
    {
        struct_to_key_value(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

fn struct_to_key_value<T: ?Sized + Serialize>(
    serializer: &mut KeyValueSerializer,
    key: &str,
    value: &T,
) -> Result<(), Serror> {
    key.serialize(&mut *serializer)?;
    serializer.output += &serializer.separator;
    value.serialize(&mut *serializer)?;
    serializer.output += "\n";
    Ok(())
}
