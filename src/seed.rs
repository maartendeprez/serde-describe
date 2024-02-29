use std::collections::BTreeMap;

use serde::de::{DeserializeSeed, Error, Visitor};
use serde_value::Value;

use crate::{EnumSchema, Schema, SimpleSchema, StructSchema, VariantSchema};

pub enum SchemaSeed<'a> {
    Simple(&'a SimpleSchema),
    Struct(StructSeed<'a>),
    Enum(EnumSeed<'a>),
}

impl<'a> SchemaSeed<'a> {
    pub fn new(schema: &'a Schema) -> Self {
        match schema {
            Schema::Simple(s) => SchemaSeed::Simple(s),
            Schema::Option(_) => todo!(),
            Schema::Seq(_) => todo!(),
            Schema::Map(_) => todo!(),
            Schema::Newtype(_) => todo!(),
            Schema::Struct(s) => SchemaSeed::Struct(StructSeed::new(s)),
            Schema::Enum(s) => SchemaSeed::Enum(EnumSeed::new(s)),
            Schema::Tuple(_) => todo!(),
        }
    }
}

impl<'de> DeserializeSeed<'de> for &SchemaSeed<'_> {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match self {
            SchemaSeed::Simple(s) => match s {
                SimpleSchema::Unit => deserializer.deserialize_unit(UnitVisitor),
                SimpleSchema::Bool => deserializer.deserialize_bool(BoolVisitor),
                SimpleSchema::U8 => deserializer.deserialize_u8(U8Visitor),
                SimpleSchema::U16 => deserializer.deserialize_u16(U16Visitor),
                SimpleSchema::U32 => deserializer.deserialize_u32(U32Visitor),
                SimpleSchema::U64 => deserializer.deserialize_u64(U64Visitor),
                SimpleSchema::U128 => unimplemented!(), // deserializer.deserialize_u128(U128Visitor),
                SimpleSchema::I8 => deserializer.deserialize_i8(I8Visitor),
                SimpleSchema::I16 => deserializer.deserialize_i16(I16Visitor),
                SimpleSchema::I32 => deserializer.deserialize_i32(I32Visitor),
                SimpleSchema::I64 => deserializer.deserialize_i64(I64Visitor),
                SimpleSchema::I128 => unimplemented!(), // deserializer.deserialize_i128(I128Visitor),
                SimpleSchema::F32 => deserializer.deserialize_f32(F32Visitor),
                SimpleSchema::F64 => deserializer.deserialize_f64(F64Visitor),
                SimpleSchema::Char => deserializer.deserialize_char(CharVisitor),
                SimpleSchema::String => deserializer.deserialize_string(StringVisitor),
                SimpleSchema::Bytes => deserializer.deserialize_bytes(BytesVisitor),
            },
            SchemaSeed::Struct(s) => s.deserialize(deserializer),
            SchemaSeed::Enum(s) => s.deserialize(deserializer),
        }
    }
}

macro_rules! simple_visitor {
    ($name:ident, $descr:expr, $visit_fn:ident, $type:ty, $arg:ident, $value:expr) => {
        struct $name;

        impl<'de> Visitor<'de> for $name {
            type Value = Value;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, $descr)
            }

            fn $visit_fn<E>(self, $arg: $type) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok($value)
            }
        }
    };
}

struct UnitVisitor;

impl<'de> Visitor<'de> for UnitVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unit")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Unit)
    }
}

simple_visitor!(BoolVisitor, "bool", visit_bool, bool, v, Value::Bool(v));
simple_visitor!(U8Visitor, "u8", visit_u8, u8, v, Value::U8(v));
simple_visitor!(U16Visitor, "u16", visit_u16, u16, v, Value::U16(v));
simple_visitor!(U32Visitor, "u32", visit_u32, u32, v, Value::U32(v));
simple_visitor!(U64Visitor, "u64", visit_u64, u64, v, Value::U64(v));
//simple_visitor!(U128Visitor, "u128", visit_u128, u128, v, Value::U128(v));
simple_visitor!(I8Visitor, "i8", visit_i8, i8, v, Value::I8(v));
simple_visitor!(I16Visitor, "i16", visit_i16, i16, v, Value::I16(v));
simple_visitor!(I32Visitor, "i32", visit_i32, i32, v, Value::I32(v));
simple_visitor!(I64Visitor, "i64", visit_i64, i64, v, Value::I64(v));
//simple_visitor!(I128Visitor, "i128", visit_i128, i128, v, Value::I128(v));
simple_visitor!(F32Visitor, "f32", visit_f32, f32, v, Value::F32(v));
simple_visitor!(F64Visitor, "f64", visit_f64, f64, v, Value::F64(v));
simple_visitor!(CharVisitor, "char", visit_char, char, v, Value::Char(v));
simple_visitor!(
    StringVisitor,
    "string",
    visit_string,
    String,
    v,
    Value::String(v)
);
simple_visitor!(
    BytesVisitor,
    "bytes",
    visit_byte_buf,
    Vec<u8>,
    v,
    Value::Bytes(v)
);

pub enum StructSeed<'a> {
    Tuple(&'a str, Vec<(&'a str, SchemaSeed<'a>)>),
}

impl<'a> StructSeed<'a> {
    pub fn new(schema: &'a StructSchema) -> Self {
        Self::Tuple(
            &schema.name,
            schema
                .fields
                .iter()
                .map(|field| (field.name.as_str(), SchemaSeed::new(&field.value)))
                .collect(),
        )
    }
}

impl<'de> DeserializeSeed<'de> for &StructSeed<'_> {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match self {
            StructSeed::Tuple(name, fields) => {
                deserializer.deserialize_tuple(fields.len(), TupleStructVisitor(name, fields))
            }
        }
    }
}

struct TupleStructVisitor<'a>(&'a str, &'a [(&'a str, SchemaSeed<'a>)]);

impl<'de> Visitor<'de> for TupleStructVisitor<'_> {
    type Value = Value;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "struct {}", self.0)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let values = self
            .1
            .iter()
            .map(|(name, seed)| {
                Ok((
                    serde_value::Value::String(name.to_string()),
                    seq.next_element_seed(seed)?
                        .ok_or_else(|| A::Error::custom("missing field"))?,
                ))
            })
            .collect::<Result<_, A::Error>>()?;
        Ok(Value::Map(values))
    }
}

pub enum EnumSeed<'a> {
    U64Tag(&'a str, Vec<(&'a str, SchemaSeed<'a>)>),
}

static UNIT_SCHEMA: Schema = Schema::Simple(SimpleSchema::Unit);

impl<'a> EnumSeed<'a> {
    pub fn new(schema: &'a EnumSchema) -> Self {
        Self::U64Tag(
            &schema.name,
            schema
                .variants
                .iter()
                .map(|variant| match variant {
                    VariantSchema::Unit(s) => (s.name.as_str(), SchemaSeed::new(&UNIT_SCHEMA)),
                    VariantSchema::Newtype(s) => (s.name.as_str(), SchemaSeed::new(&s.value)),
                    VariantSchema::Tuple(s) => (
                        s.name.as_str(),
                        SchemaSeed::Struct(StructSeed::Tuple(
                            "tuple variant",
                            s.values
                                .iter()
                                .enumerate()
                                .map(|(i, s)| (&"field"[0..i], SchemaSeed::new(s)))
                                .collect(),
                        )),
                    ),
                    VariantSchema::Struct(s) => (
                        s.name.as_str(),
                        SchemaSeed::Struct(StructSeed::Tuple(
                            "tuple variant",
                            s.fields
                                .iter()
                                .enumerate()
                                .map(|(i, f)| (f.name.as_str(), SchemaSeed::new(&f.value)))
                                .collect(),
                        )),
                    ),
                })
                .collect(),
        )
    }
}

impl<'de> DeserializeSeed<'de> for &EnumSeed<'_> {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match self {
            EnumSeed::U64Tag(name, variants) => {
                deserializer.deserialize_tuple(2, U64EnumVisitor(name, variants))
            }
        }
    }
}

struct U64EnumVisitor<'a>(&'a str, &'a [(&'a str, SchemaSeed<'a>)]);

impl<'de> Visitor<'de> for U64EnumVisitor<'_> {
    type Value = Value;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "enum {}", self.0)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let tag = seq
            .next_element::<u64>()?
            .ok_or_else(|| A::Error::custom("missing tag"))?;
        let (name, seed) = self
            .1
            .get(tag as usize)
            .ok_or_else(|| A::Error::custom(format!("invalid variant: {tag}")))?;
        let value = seq
            .next_element_seed(seed)?
            .ok_or_else(|| A::Error::custom("missing value"))?;
        Ok(Value::Map(BTreeMap::from_iter([(
            Value::String(name.to_string()),
            value,
        )])))
    }
}
