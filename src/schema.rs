use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum SchemaItem {
    Schema(Box<Schema>),
    Named(SchemaName),
}

#[derive(SerializeDisplay, DeserializeFromStr, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct SchemaName(String, Vec<SchemaName>);

impl SchemaName {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self(name.into(), Vec::new())
    }

    pub fn argument(mut self, arg: SchemaName) -> Self {
        self.1.push(arg);
        self
    }
}

impl Display for SchemaName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        let mut args = self.1.iter();
        if let Some(first) = args.next() {
            write!(f, "<{first}")?;
            args.try_for_each(|arg| write!(f, ", {arg}"))?;
            write!(f, ">")?;
        }
        Ok(())
    }
}

impl FromStr for SchemaName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl From<SchemaName> for SchemaItem {
    fn from(name: SchemaName) -> Self {
        SchemaItem::Named(name)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum Schema {
    Simple(SimpleSchema),
    Option(OptionSchema),
    Tuple(TupleSchema),
    Seq(SeqSchema),
    Map(MapSchema),
    Struct(StructSchema),
    Enum(EnumSchema),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum Expected<'a> {
    Simple(SimpleSchema),
    Option,
    Tuple(usize),
    Seq,
    Map,
    Struct(&'a str),
    Enum(&'a str),
}

impl Schema {
    pub fn expected(&self) -> Expected {
        match self {
            Schema::Simple(s) => Expected::Simple(*s),
            Schema::Option(_) => Expected::Option,
            Schema::Tuple(s) => Expected::Tuple(s.values.len()),
            Schema::Seq(_) => Expected::Seq,
            Schema::Map(_) => Expected::Map,
            Schema::Struct(s) => Expected::Struct(&s.name),
            Schema::Enum(s) => Expected::Enum(&s.name),
        }
    }
}

impl Display for Expected<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expected::Simple(s) => write!(f, "{s}"),
            Expected::Option => write!(f, "option"),
            Expected::Tuple(n) => write!(f, "{n}-element tuple"),
            Expected::Seq => write!(f, "sequence"),
            Expected::Map => write!(f, "map"),
            Expected::Struct(name) => write!(f, "struct {name}"),
            Expected::Enum(name) => write!(f, "enum {name}"),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum SimpleSchema {
    Unit,
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    Char,
    String,
    Bytes,
}

impl Display for SimpleSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimpleSchema::Unit => write!(f, "unit"),
            SimpleSchema::Bool => write!(f, "bool"),
            SimpleSchema::U8 => write!(f, "u8"),
            SimpleSchema::U16 => write!(f, "u16"),
            SimpleSchema::U32 => write!(f, "u32"),
            SimpleSchema::U64 => write!(f, "u64"),
            SimpleSchema::U128 => write!(f, "u128"),
            SimpleSchema::I8 => write!(f, "i8"),
            SimpleSchema::I16 => write!(f, "i16"),
            SimpleSchema::I32 => write!(f, "i32"),
            SimpleSchema::I64 => write!(f, "i64"),
            SimpleSchema::I128 => write!(f, "i128"),
            SimpleSchema::F32 => write!(f, "f32"),
            SimpleSchema::F64 => write!(f, "f64"),
            SimpleSchema::Char => write!(f, "char"),
            SimpleSchema::String => write!(f, "string"),
            SimpleSchema::Bytes => write!(f, "bytes"),
        }
    }
}

impl From<SimpleSchema> for Schema {
    fn from(value: SimpleSchema) -> Self {
        Schema::Simple(value)
    }
}

impl From<SimpleSchema> for SchemaItem {
    fn from(value: SimpleSchema) -> Self {
        SchemaItem::Schema(Box::new(value.into()))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct OptionSchema {
    pub(crate) value: SchemaItem,
}

impl OptionSchema {
    pub fn new<V: Into<SchemaItem>>(value: V) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl From<OptionSchema> for Schema {
    fn from(value: OptionSchema) -> Self {
        Schema::Option(value)
    }
}

impl From<OptionSchema> for SchemaItem {
    fn from(value: OptionSchema) -> Self {
        SchemaItem::Schema(Box::new(value.into()))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct SeqSchema {
    pub(crate) value: SchemaItem,
}

impl SeqSchema {
    pub fn new<V: Into<SchemaItem>>(value: V) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl From<SeqSchema> for Schema {
    fn from(value: SeqSchema) -> Self {
        Schema::Seq(value)
    }
}

impl From<SeqSchema> for SchemaItem {
    fn from(value: SeqSchema) -> Self {
        SchemaItem::Schema(Box::new(value.into()))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct MapSchema {
    pub(crate) key: SchemaItem,
    pub(crate) value: SchemaItem,
}

impl MapSchema {
    pub fn new<K: Into<SchemaItem>, V: Into<SchemaItem>>(key: K, value: V) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

impl From<MapSchema> for Schema {
    fn from(value: MapSchema) -> Self {
        Schema::Map(value)
    }
}

impl From<MapSchema> for SchemaItem {
    fn from(value: MapSchema) -> Self {
        SchemaItem::Schema(Box::new(value.into()))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct TupleSchema {
    pub(crate) values: Vec<SchemaItem>,
}

impl TupleSchema {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn element<V: Into<SchemaItem>>(mut self, value: V) -> Self {
        self.values.push(value.into());
        self
    }
}

impl From<TupleSchema> for Schema {
    fn from(value: TupleSchema) -> Self {
        Schema::Tuple(value)
    }
}

impl From<TupleSchema> for SchemaItem {
    fn from(value: TupleSchema) -> Self {
        SchemaItem::Schema(Box::new(value.into()))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct StructSchema {
    pub(crate) name: String,
    pub(crate) fields: FieldsSchema,
    #[serde(default, skip_serializing_if = "is_default")]
    pub(crate) rename: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub(crate) rename_all: Option<RenameAll>,
}

impl StructSchema {
    pub fn new<T: Into<String>, V: Into<FieldsSchema>>(name: T, fields: V) -> Self {
        Self {
            name: name.into(),
            fields: fields.into(),
            rename: None,
            rename_all: None,
        }
    }
}

impl From<StructSchema> for Schema {
    fn from(value: StructSchema) -> Self {
        Schema::Struct(value)
    }
}

impl From<StructSchema> for SchemaItem {
    fn from(value: StructSchema) -> Self {
        SchemaItem::Schema(Box::new(value.into()))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum FieldsSchema {
    Tuple(TupleSchema),
    Named(NamedFieldsSchema),
}

impl From<TupleSchema> for FieldsSchema {
    fn from(value: TupleSchema) -> Self {
        Self::Tuple(value)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct NamedFieldsSchema {
    fields: Vec<NamedFieldSchema>,
}

impl NamedFieldsSchema {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn field(mut self, field: NamedFieldSchema) -> Self {
        self.fields.push(field);
        self
    }
}

impl From<NamedFieldsSchema> for FieldsSchema {
    fn from(value: NamedFieldsSchema) -> Self {
        Self::Named(value)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct NamedFieldSchema {
    pub(crate) name: String,
    pub(crate) value: SchemaItem,
    #[serde(default, skip_serializing_if = "is_default")]
    pub(crate) rename: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub(crate) aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub(crate) default: Option<serde_value::Value>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub(crate) flatten: bool,
}

impl NamedFieldSchema {
    pub fn new<T: Into<String>, V: Into<SchemaItem>>(name: T, value: V) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            rename: None,
            aliases: Vec::new(),
            default: None,
            flatten: false,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct EnumSchema {
    pub(crate) name: String,
    pub(crate) variants: Vec<VariantSchema>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub(crate) repr: EnumRepr,
    #[serde(default, skip_serializing_if = "is_default")]
    pub(crate) rename: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub(crate) rename_all: Option<RenameAll>,
}

impl EnumSchema {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            variants: Vec::new(),
            repr: EnumRepr::ExternallyTagged,
            rename: None,
            rename_all: None,
        }
    }

    pub fn variant(mut self, variant: VariantSchema) -> Self {
        self.variants.push(variant);
        self
    }
}

impl From<EnumSchema> for Schema {
    fn from(value: EnumSchema) -> Self {
        Schema::Enum(value)
    }
}

impl From<EnumSchema> for SchemaItem {
    fn from(value: EnumSchema) -> Self {
        SchemaItem::Schema(Box::new(value.into()))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct VariantSchema {
    pub(crate) name: String,
    pub(crate) fields: FieldsSchema,
    #[serde(default, skip_serializing_if = "is_default")]
    pub(crate) id: Option<usize>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub(crate) rename: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub(crate) aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub(crate) other: bool,
}

impl VariantSchema {
    pub fn new<T: Into<String>, V: Into<FieldsSchema>>(name: T, fields: V) -> Self {
        Self {
            name: name.into(),
            fields: fields.into(),
            id: None,
            rename: None,
            aliases: Vec::new(),
            other: false,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Default, Clone, Debug)]
pub enum EnumRepr {
    #[default]
    ExternallyTagged,
    InternallyTagged {
        tag: String,
    },
    AdjacentlyTagged {
        tag: String,
        content: String,
    },
    //Untagged,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum RenameAll {
    #[serde(rename = "lowercase")]
    Lower,
    #[serde(rename = "UPPERCASE")]
    Upper,
    #[serde(rename = "PascalCase")]
    Pascal,
    #[serde(rename = "camelCase")]
    Camel,
    #[serde(rename = "snake_case")]
    Snake,
    #[serde(rename = "SCREAMING_SNAKE_CASE")]
    ScreamingSnake,
    #[serde(rename = "kebab-case")]
    Kebab,
    #[serde(rename = "SCREAMING-KEBAB-CASE")]
    ScreamingKebab,
}

pub(crate) fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}
