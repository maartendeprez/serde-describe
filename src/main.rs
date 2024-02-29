// use bincode::Options;
use serde::{Deserialize, Serialize};
use serde_descr::{
    Describe, Description, EnumSchema, NamedFieldSchema, NamedFieldsSchema, SchemaName,
    StructSchema, TupleSchema, VariantSchema,
};

fn main() {
    let my_struct_schema = MyStruct::<MyEnum>::describe();

    // let value = MyStruct {
    //     string: String::from("another"),
    //     int: 42,
    //     r#enum: MyEnum::StructVariant { float_field: 42.1 },
    // };

    // let options = bincode::DefaultOptions::new();
    // let bytes = options.serialize(&value).unwrap();
    // let value1 = options
    //     .deserialize_seed(&SchemaSeed::new(&my_struct_schema), &bytes)
    //     .unwrap();

    // let bytes = serde_json::to_vec(&value2).unwrap();
    // let value1 = serde_json::from_slice::<MyStruct>(&bytes).unwrap();

    println!(
        "Schema: {}",
        serde_json::to_string_pretty(&my_struct_schema).unwrap()
    );
}

#[derive(Serialize, Deserialize)]
struct MyStruct<T> {
    string: String,
    int: u64,
    r#enum: T,
}

#[derive(Serialize, Deserialize)]
struct MyStruct2 {
    new_field: String,
    string: String,
    int: u64,
    r#enum: MyEnum,
}

#[derive(Serialize, Deserialize)]
enum MyEnum {
    UnitVariant,
    NewtypeVariant(String),
    TupleVariant(String, u64),
    StructVariant { float_field: f64 },
}

impl<T: Describe> Describe for MyStruct<T> {
    fn schema_name() -> SchemaName {
        SchemaName::new("MyStruct").argument(T::schema_name())
    }

    fn add_schema(map: &mut Description) {
        map.add(Self::schema_name(), || {
            StructSchema::new(
                "MyStruct",
                NamedFieldsSchema::new()
                    .field(NamedFieldSchema::new("string", String::schema()))
                    .field(NamedFieldSchema::new("int", u64::schema()))
                    .field(NamedFieldSchema::new("enum", T::schema())),
            )
        });
        T::add_schema(map);
    }
}

impl Describe for MyStruct2 {
    fn schema_name() -> SchemaName {
        SchemaName::new("MyStruct2")
    }

    fn add_schema(map: &mut Description) {
        map.add(Self::schema_name(), || {
            StructSchema::new(
                "MyStruct2",
                NamedFieldsSchema::new()
                    .field(NamedFieldSchema::new("new_field", String::schema()))
                    .field(NamedFieldSchema::new("string", String::schema()))
                    .field(NamedFieldSchema::new("int", u64::schema()))
                    .field(NamedFieldSchema::new("enum", MyEnum::schema())),
            )
        });
        MyEnum::add_schema(map);
    }
}

impl Describe for MyEnum {
    fn schema_name() -> SchemaName {
        SchemaName::new("MyEnum")
    }

    fn add_schema(map: &mut Description) {
        map.add(Self::schema_name(), || {
            EnumSchema::new("MyEnum")
                .variant(VariantSchema::new("UnitVariant", TupleSchema::new()))
                .variant(VariantSchema::new(
                    "NewtypeVariant",
                    TupleSchema::new().element(String::schema()),
                ))
                .variant(VariantSchema::new(
                    "TupleVariant",
                    TupleSchema::new()
                        .element(String::schema())
                        .element(u64::schema()),
                ))
                .variant(VariantSchema::new(
                    "StructVariant",
                    NamedFieldsSchema::new()
                        .field(NamedFieldSchema::new("float_field", f64::schema())),
                ))
        });
    }
}
