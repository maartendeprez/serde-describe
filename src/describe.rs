use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
};

use serde::{Deserialize, Serialize};

use crate::{is_default, MapSchema, Schema, SchemaItem, SchemaName, SeqSchema, SimpleSchema};

pub trait Describe {
    fn schema_name() -> SchemaName;

    fn schema() -> SchemaItem {
        Self::schema_name().into()
    }

    fn add_schema(_map: &mut Description) {}

    fn describe() -> Description {
        let mut map = Description::new(Self::schema());
        Self::add_schema(&mut map);
        map
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct Description {
    pub(crate) schema: SchemaItem,
    #[serde(default, skip_serializing_if = "is_default")]
    pub(crate) items: BTreeMap<SchemaName, Schema>,
}

impl Description {
    pub fn new(schema: SchemaItem) -> Self {
        Self {
            schema,
            items: BTreeMap::new(),
        }
    }

    pub fn add<F: FnOnce() -> R, R: Into<Schema>>(&mut self, name: SchemaName, schema: F) {
        self.items.entry(name).or_insert_with(|| schema().into());
    }
}

impl Describe for () {
    fn schema_name() -> SchemaName {
        SchemaName::new("unit")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::Unit.into()
    }
}

impl Describe for bool {
    fn schema_name() -> SchemaName {
        SchemaName::new("bool")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::Bool.into()
    }
}

impl Describe for u8 {
    fn schema_name() -> SchemaName {
        SchemaName::new("u8")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::U8.into()
    }
}

impl Describe for u16 {
    fn schema_name() -> SchemaName {
        SchemaName::new("u16")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::U16.into()
    }
}

impl Describe for u32 {
    fn schema_name() -> SchemaName {
        SchemaName::new("u32")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::U32.into()
    }
}

impl Describe for u64 {
    fn schema_name() -> SchemaName {
        SchemaName::new("u64")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::U64.into()
    }
}

impl Describe for usize {
    fn schema_name() -> SchemaName {
        SchemaName::new("usize")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::U64.into()
    }
}

impl Describe for u128 {
    fn schema_name() -> SchemaName {
        SchemaName::new("u128")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::U128.into()
    }
}

impl Describe for i8 {
    fn schema_name() -> SchemaName {
        SchemaName::new("i8")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::I8.into()
    }
}

impl Describe for i16 {
    fn schema_name() -> SchemaName {
        SchemaName::new("i16")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::I16.into()
    }
}

impl Describe for i32 {
    fn schema_name() -> SchemaName {
        SchemaName::new("i32")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::I32.into()
    }
}

impl Describe for i64 {
    fn schema_name() -> SchemaName {
        SchemaName::new("i64")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::I64.into()
    }
}

impl Describe for isize {
    fn schema_name() -> SchemaName {
        SchemaName::new("isize")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::I64.into()
    }
}

impl Describe for i128 {
    fn schema_name() -> SchemaName {
        SchemaName::new("i128")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::I128.into()
    }
}

impl Describe for f32 {
    fn schema_name() -> SchemaName {
        SchemaName::new("f32")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::F32.into()
    }
}

impl Describe for f64 {
    fn schema_name() -> SchemaName {
        SchemaName::new("f64")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::F64.into()
    }
}

impl Describe for String {
    fn schema_name() -> SchemaName {
        SchemaName::new("str")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::String.into()
    }
}

impl Describe for str {
    fn schema_name() -> SchemaName {
        SchemaName::new("str")
    }

    fn schema() -> SchemaItem {
        SimpleSchema::String.into()
    }
}

// impl Describe for Vec<u8> {
//     fn schema_name() -> SchemaName {
//         SchemaName::new("bytes")
//     }

//     fn schema() -> SchemaItem {
//         SimpleSchema::Bytes.into()
//     }
// }

impl<T: Describe> Describe for Vec<T> {
    fn schema_name() -> SchemaName {
        SchemaName::new("std::vec::Vec").argument(T::schema_name())
    }

    fn schema() -> SchemaItem {
        SeqSchema::new(T::schema()).into()
    }

    fn add_schema(map: &mut Description) {
        T::add_schema(map);
    }
}

impl<T: Describe> Describe for [T] {
    fn schema_name() -> SchemaName {
        SchemaName::new("std::slice").argument(T::schema_name())
    }

    fn schema() -> SchemaItem {
        SeqSchema::new(T::schema()).into()
    }

    fn add_schema(map: &mut Description) {
        T::add_schema(map);
    }
}

impl<T: Describe> Describe for BTreeSet<T> {
    fn schema_name() -> SchemaName {
        SchemaName::new("std::collections::BTreeSet").argument(T::schema_name())
    }

    fn schema() -> SchemaItem {
        SeqSchema::new(T::schema()).into()
    }

    fn add_schema(map: &mut Description) {
        T::add_schema(map);
    }
}

impl<T: Describe> Describe for HashSet<T> {
    fn schema_name() -> SchemaName {
        SchemaName::new("std::collections::HashSet").argument(T::schema_name())
    }

    fn schema() -> SchemaItem {
        SeqSchema::new(T::schema()).into()
    }

    fn add_schema(map: &mut Description) {
        T::add_schema(map);
    }
}

impl<K: Describe, V: Describe> Describe for BTreeMap<K, V> {
    fn schema_name() -> SchemaName {
        SchemaName::new("std::collections::BTreeMap")
            .argument(K::schema_name())
            .argument(V::schema_name())
    }

    fn schema() -> SchemaItem {
        MapSchema::new(K::schema(), V::schema()).into()
    }

    fn add_schema(map: &mut Description) {
        K::add_schema(map);
        V::add_schema(map);
    }
}

impl<K: Describe, V: Describe> Describe for HashMap<K, V> {
    fn schema_name() -> SchemaName {
        SchemaName::new("std::collections::HashMap")
            .argument(K::schema_name())
            .argument(V::schema_name())
    }

    fn schema() -> SchemaItem {
        MapSchema::new(K::schema(), V::schema()).into()
    }

    fn add_schema(map: &mut Description) {
        K::add_schema(map);
        V::add_schema(map);
    }
}

impl<T: Describe> Describe for &T {
    fn schema_name() -> SchemaName {
        T::schema_name()
    }

    fn schema() -> SchemaItem {
        T::schema()
    }

    fn add_schema(map: &mut Description) {
        T::add_schema(map)
    }
}

impl<T: Describe> Describe for &mut T {
    fn schema_name() -> SchemaName {
        T::schema_name()
    }

    fn schema() -> SchemaItem {
        T::schema()
    }

    fn add_schema(map: &mut Description) {
        T::add_schema(map)
    }
}

impl<T: Describe> Describe for Box<T> {
    fn schema_name() -> SchemaName {
        T::schema_name()
    }

    fn schema() -> SchemaItem {
        T::schema()
    }

    fn add_schema(map: &mut Description) {
        T::add_schema(map)
    }
}

impl<T: Describe> Describe for Rc<T> {
    fn schema_name() -> SchemaName {
        T::schema_name()
    }

    fn schema() -> SchemaItem {
        T::schema()
    }

    fn add_schema(map: &mut Description) {
        T::add_schema(map)
    }
}

impl<T: Describe> Describe for Arc<T> {
    fn schema_name() -> SchemaName {
        T::schema_name()
    }

    fn schema() -> SchemaItem {
        T::schema()
    }

    fn add_schema(map: &mut Description) {
        T::add_schema(map)
    }
}

impl<T: Describe + Clone> Describe for Cow<'_, T> {
    fn schema_name() -> SchemaName {
        T::schema_name()
    }

    fn schema() -> SchemaItem {
        T::schema()
    }

    fn add_schema(map: &mut Description) {
        T::add_schema(map)
    }
}

impl<T: Describe> Describe for Mutex<T> {
    fn schema_name() -> SchemaName {
        T::schema_name()
    }

    fn schema() -> SchemaItem {
        T::schema()
    }

    fn add_schema(map: &mut Description) {
        T::add_schema(map)
    }
}

impl<T: Describe> Describe for RwLock<T> {
    fn schema_name() -> SchemaName {
        T::schema_name()
    }

    fn schema() -> SchemaItem {
        T::schema()
    }

    fn add_schema(map: &mut Description) {
        T::add_schema(map)
    }
}
