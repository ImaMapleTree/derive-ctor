use derive_ctor::ctor;

#[derive(ctor, Debug, PartialEq)]
struct StructValue {
    provided: u32,
    #[ctor(value(10))]
    generated: u32
}

#[derive(ctor, Debug, PartialEq)]
struct StructManyValue {
    provided: u32,
    #[ctor(value(11))]
    generated1: u32,
    #[ctor(value(false))]
    generated2: bool
}

#[derive(ctor, Debug, PartialEq)]
struct StructComplexValue {
    provided: u32,
    #[ctor(value(String::from("Foo")))]
    generated: String
}

#[derive(ctor, Debug, PartialEq)]
struct StructReliantValue {
    provided: u32,
    #[ctor(value(provided.to_string()))]
    generated: String
}

#[test]
fn test_struct_value_field() {
    let test = StructValue::new(100);
    assert_eq!(StructValue { provided: 100, generated: 10 }, test);
}

#[test]
fn test_struct_many_value_fields() {
    let test = StructManyValue::new(101);
    assert_eq!(StructManyValue { provided: 101, generated1: 11, generated2: false }, test);
}

#[test]
fn test_struct_complex_value_field() {
    let test = StructComplexValue::new(102);
    assert_eq!(StructComplexValue { provided: 102, generated: String::from("Foo") }, test);
}

#[test]
fn test_struct_reliant_value_field() {
    let test = StructReliantValue::new(103);
    assert_eq!(StructReliantValue { provided: 103, generated: 103.to_string() }, test);
}