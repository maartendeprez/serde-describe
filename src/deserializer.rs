use std::collections::BTreeMap;

use serde::de::{
    value::{StrDeserializer, StringDeserializer},
    DeserializeSeed, Deserializer, Error, MapAccess, SeqAccess, Visitor,
};

use crate::{
    schema::EnumRepr, Description, Schema, SchemaItem, SchemaName, SimpleSchema, VariantSchema,
};

pub struct SchemaDeserializer<'a, 'b, D> {
    schema: &'a SchemaDecode<'b>,
    items: &'a SchemaDecodeItems<'b>,
    deserializer: D,
}

pub struct SchemaDecoder<'a> {
    schema: SchemaDecodeItem<'a>,
    items: SchemaDecodeItems<'a>,
}

type SchemaDecodeItems<'a> = BTreeMap<&'a SchemaName, SchemaDecode<'a>>;

pub enum SchemaDecodeItem<'a> {
    Decode(Box<SchemaDecode<'a>>),
    Named(&'a SchemaName),
}

pub enum SchemaDecode<'a> {
    Simple(SimpleSchema),
    Option(OptionDecode<'a>),
    Tuple(TupleDecode<'a>),
    Seq(SeqDecode<'a>),
    Map(MapDecode<'a>),
    Struct(StructDeserializer<'a>),
    Enum(EnumDeserializer<'a>),
}

pub struct DeserializerOptions {
    enum_format: EnumFormat,
    struct_format: StructFormat,
    borrowing: bool,
}

enum EnumFormat {
    Tuple,
    Map,
}

enum StructFormat {
    Tuple,
    Map,
}

impl<'a, 'b: 'a, D> SchemaDeserializer<'a, 'b, D> {
    pub fn new(decoder: &'a SchemaDecoder<'b>, deserializer: D) -> Self {
        Self {
            schema: decoder.schema.lookup(&decoder.items),
            items: &decoder.items,
            deserializer,
        }
    }
}

impl<'a> SchemaDecoder<'a> {
    pub fn new(descr: &'a Description) -> Self {
        Self {
            schema: SchemaDecodeItem::new(&descr.schema),
            items: descr
                .items
                .iter()
                .map(|(name, schema)| (name, SchemaDecode::new(schema)))
                .collect(),
        }
    }
}

impl<'a> SchemaDecodeItem<'a> {
    fn new(schema: &'a SchemaItem) -> Self {
        match schema {
            SchemaItem::Schema(schema) => Self::Decode(Box::new(SchemaDecode::new(schema))),
            SchemaItem::Named(name) => Self::Named(name),
        }
    }

    fn lookup<'b>(&'b self, items: &'b SchemaDecodeItems<'a>) -> &'b SchemaDecode<'a> {
        match self {
            SchemaDecodeItem::Decode(decode) => decode,
            SchemaDecodeItem::Named(name) => items.get(name).unwrap(),
        }
    }
}

impl<'a> SchemaDecode<'a> {
    fn new(schema: &'a Schema) -> Self {
        todo!()
    }
}

impl DeserializerOptions {
    pub fn text() -> Self {
        DeserializerOptions {
            enum_format: EnumFormat::Map,
            struct_format: StructFormat::Map,
            borrowing: false,
        }
    }

    pub fn binary() -> Self {
        DeserializerOptions {
            enum_format: EnumFormat::Tuple,
            struct_format: StructFormat::Tuple,
            borrowing: false,
        }
    }

    pub fn borrowing(mut self) -> Self {
        self.borrowing = true;
        self
    }
}

struct SchemaSeed<'a, 'b: 'a, T> {
    schema: &'a SchemaDecode<'b>,
    items: &'a SchemaDecodeItems<'b>,
    seed: T,
}

impl<'de, T: DeserializeSeed<'de>> DeserializeSeed<'de> for SchemaSeed<'_, '_, T> {
    type Value = T::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.seed.deserialize(SchemaDeserializer {
            schema: self.schema,
            items: self.items,
            deserializer,
        })
    }
}

impl<'de, D: Deserializer<'de>> Deserializer<'de> for SchemaDeserializer<'_, '_, D> {
    type Error = D::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            SchemaDecode::Simple(s) => match s {
                SimpleSchema::Unit => self.deserializer.deserialize_unit(visitor),
                SimpleSchema::Bool => self.deserializer.deserialize_bool(visitor),
                SimpleSchema::U8 => self.deserializer.deserialize_u8(visitor),
                SimpleSchema::U16 => self.deserializer.deserialize_u16(visitor),
                SimpleSchema::U32 => self.deserializer.deserialize_u32(visitor),
                SimpleSchema::U64 => self.deserializer.deserialize_u64(visitor),
                SimpleSchema::U128 => self.deserializer.deserialize_u128(visitor),
                SimpleSchema::I8 => self.deserializer.deserialize_i8(visitor),
                SimpleSchema::I16 => self.deserializer.deserialize_i16(visitor),
                SimpleSchema::I32 => self.deserializer.deserialize_i32(visitor),
                SimpleSchema::I64 => self.deserializer.deserialize_i64(visitor),
                SimpleSchema::I128 => self.deserializer.deserialize_i128(visitor),
                SimpleSchema::F32 => self.deserializer.deserialize_f32(visitor),
                SimpleSchema::F64 => self.deserializer.deserialize_f64(visitor),
                SimpleSchema::Char => self.deserializer.deserialize_char(visitor),
                SimpleSchema::String => match self.opts.borrowing {
                    true => self.deserializer.deserialize_str(visitor),
                    false => self.deserializer.deserialize_string(visitor),
                },
                SimpleSchema::Bytes => match self.opts.borrowing {
                    true => self.deserializer.deserialize_bytes(visitor),
                    false => self.deserializer.deserialize_byte_buf(visitor),
                },
            },
            Schema::Option(s) => self.deserializer.deserialize_option(OptionVisitor {
                value: &s.value,
                opts: self.opts,
                visitor,
            }),
            Schema::Seq(s) => self.deserializer.deserialize_seq(SeqVisitor {
                value: &s.value,
                opts: self.opts,
                visitor,
            }),
            Schema::Map(s) => self.deserializer.deserialize_map(MapVisitor {
                key: &s.key,
                value: &s.value,
                opts: self.opts,
                visitor,
            }),
            Schema::Newtype(s) => visitor.visit_newtype_struct(self),
            Schema::Struct(s) => match self.opts.struct_format {
                StructFormat::Tuple => self.deserializer.deserialize_tuple(
                    s.fields.len(),
                    TupleStructVisitor {
                        fields: &s.fields,
                        opts: self.opts,
                        visitor,
                    },
                ),
                StructFormat::Map => self.deserializer.deserialize_map(MapStructVisitor {
                    fields: &s.fields,
                    opts: self.opts,
                    visitor,
                }),
            },
            Schema::Enum(s) => match self.opts.enum_format {
                EnumFormat::Tuple => self.deserializer.deserialize_tuple(
                    2,
                    TupleEnumVisitor {
                        variants: &s.variants,
                        opts: self.opts,
                        visitor,
                    },
                ),
                EnumFormat::Map => match s.repr {
                    EnumRepr::ExternallyTagged => {
                        self.deserializer
                            .deserialize_map(ExternallyTaggedEnumVisitor {
                                variants: &s.variants,
                                opts: self.opts,
                                visitor,
                            })
                    }
                    EnumRepr::InternallyTagged { tag } => {
                        todo!()
                        //self.deserializer.deserialize_map(InternallyTaggedEnumVisitor(tag, &s.variants, visitor)),
                    }
                    EnumRepr::AdjacentlyTagged { tag, content } => {
                        todo!()
                        // 		self.deserializer.deserialize_map(AdjacentlyTaggedEnumVisitor(
                        //     tag,
                        //     content,
                        //     &s.variants,
                        //     visitor,
                        // )),
                    }
                },
            },
            Schema::Tuple(s) => self.deserializer.deserialize_tuple(
                s.values.len(),
                TupleVisitor {
                    values: &s.values,
                    opts: self.opts,
                    visitor,
                },
            ),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::Bool) => self.deserializer.deserialize_bool(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::Bool)
            ))),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::I8) => self.deserializer.deserialize_i8(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::I8)
            ))),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::I8) => self.deserializer.deserialize_i8(visitor),
            Schema::Simple(SimpleSchema::I16) => self.deserializer.deserialize_i16(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::I16)
            ))),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::I8) => self.deserializer.deserialize_i8(visitor),
            Schema::Simple(SimpleSchema::I16) => self.deserializer.deserialize_i16(visitor),
            Schema::Simple(SimpleSchema::I32) => self.deserializer.deserialize_i32(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::I32)
            ))),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::I8) => self.deserializer.deserialize_i8(visitor),
            Schema::Simple(SimpleSchema::I16) => self.deserializer.deserialize_i16(visitor),
            Schema::Simple(SimpleSchema::I32) => self.deserializer.deserialize_i32(visitor),
            Schema::Simple(SimpleSchema::I64) => self.deserializer.deserialize_i64(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::I64)
            ))),
        }
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::I8) => self.deserializer.deserialize_i8(visitor),
            Schema::Simple(SimpleSchema::I16) => self.deserializer.deserialize_i16(visitor),
            Schema::Simple(SimpleSchema::I32) => self.deserializer.deserialize_i32(visitor),
            Schema::Simple(SimpleSchema::I64) => self.deserializer.deserialize_i64(visitor),
            Schema::Simple(SimpleSchema::I128) => self.deserializer.deserialize_i128(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::I128)
            ))),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::U8) => self.deserializer.deserialize_u8(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::U8)
            ))),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::U8) => self.deserializer.deserialize_u8(visitor),
            Schema::Simple(SimpleSchema::U16) => self.deserializer.deserialize_u16(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::U16)
            ))),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::U8) => self.deserializer.deserialize_u8(visitor),
            Schema::Simple(SimpleSchema::U16) => self.deserializer.deserialize_u16(visitor),
            Schema::Simple(SimpleSchema::U32) => self.deserializer.deserialize_u32(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::U32)
            ))),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::U8) => self.deserializer.deserialize_u8(visitor),
            Schema::Simple(SimpleSchema::U16) => self.deserializer.deserialize_u16(visitor),
            Schema::Simple(SimpleSchema::U32) => self.deserializer.deserialize_u32(visitor),
            Schema::Simple(SimpleSchema::U64) => self.deserializer.deserialize_u64(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::U64)
            ))),
        }
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::U8) => self.deserializer.deserialize_u8(visitor),
            Schema::Simple(SimpleSchema::U16) => self.deserializer.deserialize_u16(visitor),
            Schema::Simple(SimpleSchema::U32) => self.deserializer.deserialize_u32(visitor),
            Schema::Simple(SimpleSchema::U64) => self.deserializer.deserialize_u64(visitor),
            Schema::Simple(SimpleSchema::U128) => self.deserializer.deserialize_u128(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::U128)
            ))),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::F32) => self.deserializer.deserialize_f32(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::F32)
            ))),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::F32) => self.deserializer.deserialize_f32(visitor),
            Schema::Simple(SimpleSchema::F64) => self.deserializer.deserialize_f64(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::F64)
            ))),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::Char) => self.deserializer.deserialize_char(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::Char)
            ))),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::String) => self.deserializer.deserialize_str(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::String)
            ))),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::String) => self.deserializer.deserialize_string(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::String)
            ))),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::Bytes) => self.deserializer.deserialize_bytes(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::Bytes)
            ))),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::Bytes) => self.deserializer.deserialize_byte_buf(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::Bytes)
            ))),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::Unit) => self.deserializer.deserialize_unit(visitor),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::Unit)
            ))),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Option(s) => self.deserializer.deserialize_option(OptionVisitor {
                value: &s.value,
                opts: self.opts,
                visitor,
            }),
            s => visitor.visit_some(self),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::Unit) => {
                self.deserializer.deserialize_unit_struct(name, visitor)
            }
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::Unit)
            ))),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Seq(s) => self.deserializer.deserialize_seq(SeqVisitor {
                value: &s.value,
                opts: self.opts,
                visitor,
            }),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Seq
            ))),
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Tuple(s) => self.deserializer.deserialize_tuple(
                s.values.len(),
                TupleVisitor {
                    values: &s.values,
                    opts: self.opts,
                    visitor,
                },
            ),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Tuple
            ))),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Tuple(s) => self.deserializer.deserialize_tuple_struct(
                name,
                s.values.len(),
                TupleVisitor {
                    values: &s.values,
                    opts: self.opts,
                    visitor,
                },
            ),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Tuple
            ))),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Map(s) => self.deserializer.deserialize_map(MapVisitor {
                key: &s.key,
                value: &s.value,
                opts: self.opts,
                visitor,
            }),
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Map
            ))),
        }
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Struct(s) => match self.opts.struct_format {
                StructFormat::Tuple => self.deserializer.deserialize_tuple(
                    s.fields.len(),
                    TupleStructVisitor {
                        fields: &s.fields,
                        opts: self.opts,
                        visitor,
                    },
                ),
                StructFormat::Map => self.deserializer.deserialize_map(MapStructVisitor {
                    fields: &s.fields,
                    opts: self.opts,
                    visitor,
                }),
            },
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Struct
            ))),
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Enum(s) => match self.opts.enum_format {
                EnumFormat::Tuple => self
                    .deserializer
                    .deserialize_tuple(2, TupleEnumVisitor(&s.variants, visitor)),
                EnumFormat::Map => match s.repr {
                    EnumRepr::ExternallyTagged => self
                        .deserializer
                        .deserialize_map(ExternallyTaggedEnumVisitor(&s.variants, visitor)),
                    EnumRepr::InternallyTagged { tag } => self
                        .deserializer
                        .deserialize_map(InternallyTaggedEnumVisitor(tag, &s.variants, visitor)),
                    EnumRepr::AdjacentlyTagged { tag, content } => self
                        .deserializer
                        .deserialize_map(AdjacentlyTaggedEnumVisitor(
                            tag,
                            content,
                            &s.variants,
                            visitor,
                        )),
                },
            },
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Enum
            ))),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.schema {
            Schema::Simple(SimpleSchema::String) => {
                self.deserializer.deserialize_identifier(visitor)
            }
            s => Err(Self::Error::custom(format!(
                "invalid type {}, expected {}",
                s.kind(),
                Kind::Simple(SimpleSchema::String)
            ))),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct OptionVisitor<'a, V> {
    value: &'a Schema,
    opts: &'a DeserializerOptions,
    visitor: V,
}

impl<'de, V: Visitor<'de>> Visitor<'de> for OptionVisitor<'_, V> {
    type Value = V::Value;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", Kind::Option)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.visitor.visit_some(SchemaDeserializer {
            schema: self.value,
            opts: self.opts,
            deserializer,
        })
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visitor.visit_none()
    }
}

struct SeqVisitor<'a, V> {
    value: &'a Schema,
    opts: &'a DeserializerOptions,
    visitor: V,
}

impl<'de, V: Visitor<'de>> Visitor<'de> for SeqVisitor<'_, V> {
    type Value = V::Value;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", Kind::Seq)
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        self.visitor.visit_seq(SchemaSeqAccess {
            schema: self.value,
            opts: self.opts,
            seq,
        })
    }
}

struct SchemaSeqAccess<'a, A> {
    schema: &'a Schema,
    opts: &'a DeserializerOptions,
    seq: A,
}

impl<'de, A: SeqAccess<'de>> SeqAccess<'de> for SchemaSeqAccess<'_, A> {
    type Error = A::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.seq.next_element_seed(SchemaSeed {
            schema: self.schema,
            opts: self.opts,
            seed,
        })
    }
}

struct MapVisitor<'a, V> {
    key: &'a Schema,
    value: &'a Schema,
    opts: &'a DeserializerOptions,
    visitor: V,
}

impl<'de, V: Visitor<'de>> Visitor<'de> for MapVisitor<'_, V> {
    type Value = V::Value;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", Kind::Seq)
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        self.visitor.visit_map(SchemaMapAccess {
            key: self.key,
            value: self.value,
            opts: self.opts,
            map,
        })
    }
}

struct SchemaMapAccess<'a, A> {
    key: &'a Schema,
    value: &'a Schema,
    opts: &'a DeserializerOptions,
    map: A,
}

impl<'de, A: MapAccess<'de>> MapAccess<'de> for SchemaMapAccess<'_, A> {
    type Error = A::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        self.map.next_key_seed(SchemaSeed {
            schema: self.key,
            opts: self.opts,
            seed,
        })
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.map.next_value_seed(SchemaSeed {
            schema: self.value,
            opts: self.opts,
            seed,
        })
    }
}

struct TupleVisitor<'a, V> {
    values: &'a [Schema],
    opts: &'a DeserializerOptions,
    visitor: V,
}

impl<'de, V: Visitor<'de>> Visitor<'de> for TupleVisitor<'_, V> {
    type Value = V::Value;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", Kind::Tuple)
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        self.visitor.visit_seq(TupleSeqAccess {
            values: self.values.iter(),
            opts: self.opts,
            seq,
        })
    }
}

struct TupleSeqAccess<'a, I, A> {
    values: I,
    opts: &'a DeserializerOptions,
    seq: A,
}

impl<'a, 'de, I: Iterator<Item = &'a Schema>, A: SeqAccess<'de>> SeqAccess<'de>
    for TupleSeqAccess<'a, I, A>
{
    type Error = A::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.values.next() {
            Some(schema) => self.seq.next_element_seed(SchemaSeed {
                schema,
                opts: self.opts,
                seed,
            }),
            None => Ok(None),
        }
    }
}

struct TupleStructVisitor<'a, V> {
    fields: &'a [FieldSchema],
    opts: &'a DeserializerOptions,
    visitor: V,
}

impl<'de, V: Visitor<'de>> Visitor<'de> for TupleStructVisitor<'_, V> {
    type Value = V::Value;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", Kind::Struct)
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        self.visitor.visit_map(TupleStructAccess {
            fields: self.fields.iter(),
            value: None,
            opts: self.opts,
            seq,
        })
    }
}

struct TupleStructAccess<'a, I, A> {
    fields: I,
    value: Option<&'a Schema>,
    opts: &'a DeserializerOptions,
    seq: A,
}

impl<'a, 'de, I: Iterator<Item = &'a FieldSchema>, A: SeqAccess<'de>> MapAccess<'de>
    for TupleStructAccess<'a, I, A>
{
    type Error = A::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.fields.next() {
            Some(field) => {
                self.value = Some(&field.value);
                Ok(Some(seed.deserialize(StrDeserializer::new(&field.name))?))
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self
            .value
            .take()
            .ok_or_else(|| A::Error::custom("invalid use of next_value_seed"))?;
        self.seq.next_element_seed(SchemaSeed {
            schema: value,
            opts: self.opts,
            seed,
        })
    }
}

struct MapStructVisitor<'a, V> {
    fields: &'a [FieldSchema],
    opts: &'a DeserializerOptions,
    visitor: V,
}

impl<'de, V: Visitor<'de>> Visitor<'de> for MapStructVisitor<'_, V> {
    type Value = V::Value;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", Kind::Struct)
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        self.visitor.visit_map(MapStructAccess {
            fields: self.fields,
            value: None,
            opts: self.opts,
            map,
        })
    }
}

struct MapStructAccess<'a, A> {
    fields: &'a [FieldSchema],
    value: Option<&'a Schema>,
    opts: &'a DeserializerOptions,
    map: A,
}

impl<'de, A: MapAccess<'de>> MapAccess<'de> for MapStructAccess<'_, A> {
    type Error = A::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.map.next_key::<String>()? {
            Some(key) => {
                let field = self
                    .fields
                    .iter()
                    .find(|field| field.name == key)
                    .ok_or_else(|| A::Error::custom("unknown field"))?;
                self.value = Some(&field.value);
                Ok(Some(seed.deserialize(StringDeserializer::new(key))?))
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self
            .value
            .take()
            .ok_or_else(|| A::Error::custom("invalid use of next_value_seed"))?;
        self.map.next_value_seed(SchemaSeed {
            schema: value,
            opts: self.opts,
            seed,
        })
    }
}

struct TupleEnumVisitor<'a, V> {
    variants: &'a [VariantSchema],
    opts: &'a DeserializerOptions,
    visitor: V,
}

impl<'de, V: Visitor<'de>> Visitor<'de> for TupleEnumVisitor<'_, V> {
    type Value = V::Value;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", Kind::Enum)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let tag = seq
            .next_element::<usize>()?
            .ok_or_else(|| A::Error::custom("missing tag"))?;
        let variant = self
            .variants
            .get(tag)
            .ok_or_else(|| A::Error::custom("invalid variant"))?;
        /* TODO: do not assume externally tagged enum */
        match variant {
            VariantSchema::Unit(s) => self.visitor.visit_map(TupleEnumAccess {
                tag: Some(&s.name),
                value: Some(todo!()),
                opts: self.opts,
                seq,
            }),
            VariantSchema::Newtype(s) => self.visitor.visit_map(TupleEnumAccess {
                tag: Some(&s.name),
                value: Some(&s.value),
                opts: self.opts,
                seq,
            }),
            VariantSchema::Tuple(s) => self.visitor.visit_map(TupleEnumAccess {
                tag: Some(&s.name),
                value: Some(todo!()),
                opts: self.opts,
                seq,
            }),
            VariantSchema::Struct(s) => self.visitor.visit_map(TupleEnumAccess {
                tag: Some(&s.name),
                value: Some(todo!()),
                opts: self.opts,
                seq,
            }),
        }
    }
}

struct TupleEnumAccess<'a, A> {
    tag: Option<&'a str>,
    value: Option<&'a Schema>,
    opts: &'a DeserializerOptions,
    seq: A,
}

impl<'de, A: SeqAccess<'de>> MapAccess<'de> for TupleEnumAccess<'_, A> {
    type Error = A::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.tag.take() {
            Some(tag) => Ok(Some(seed.deserialize(StrDeserializer::new(tag))?)),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self
            .value
            .take()
            .ok_or_else(|| A::Error::custom("invalid use of next_value_seed"))?;
        self.seq.next_element_seed(SchemaSeed {
            schema: value,
            opts: self.opts,
            seed,
        })
    }
}

struct ExternallyTaggedEnumVisitor<'a, V> {
    variants: &'a [VariantSchema],
    opts: &'a DeserializerOptions,
    visitor: V,
}

impl<'de, V: Visitor<'de>> Visitor<'de> for ExternallyTaggedEnumVisitor<'_, V> {
    type Value = V::Value;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", Kind::Enum)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let tag = map
            .next_key::<String>()?
            .ok_or_else(|| A::Error::custom("missing tag"))?;
        let variant = self
            .variants
            .iter()
            .find(|variant| match variant {
                VariantSchema::Unit(s) => s.name == tag,
                VariantSchema::Newtype(s) => s.name == tag,
                VariantSchema::Tuple(s) => s.name == tag,
                VariantSchema::Struct(s) => s.name == tag,
            })
            .ok_or_else(|| A::Error::custom("invalid variant"))?;
        match variant {
            VariantSchema::Unit(_) => todo!(),
            VariantSchema::Newtype(s) => self.visitor.visit_map(ExternallyTaggedEnumAccess {
                tag: Some(&s.name),
                value: Some(&s.value),
                opts: self.opts,
                map,
            }),
            VariantSchema::Tuple(_) => todo!(),
            VariantSchema::Struct(_) => todo!(),
        }
    }
}

struct ExternallyTaggedEnumAccess<'a, A> {
    tag: Option<&'a str>,
    value: Option<&'a Schema>,
    opts: &'a DeserializerOptions,
    map: A,
}

impl<'de, A: MapAccess<'de>> MapAccess<'de> for ExternallyTaggedEnumAccess<'_, A> {
    type Error = A::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.tag.take() {
            Some(tag) => Ok(Some(seed.deserialize(StrDeserializer::new(tag))?)),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self
            .value
            .take()
            .ok_or_else(|| A::Error::custom("invalid use of next_value_seed"))?;
        self.map.next_value_seed(SchemaSeed {
            schema: value,
            opts: self.opts,
            seed,
        })
    }
}
