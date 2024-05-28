use derive_ctor::ctor;

#[derive(ctor, Debug, PartialEq)]
pub struct Unit;

#[test]
fn test_unit_struct() {
    let unit = Unit::new();
    assert_eq!(Unit, unit);
}

#[derive(ctor, Debug, PartialEq)]
pub struct Empty {}

#[test]
fn test_empty_struct_no_config() {
    let empty = Empty::new();
    assert_eq!(Empty { }, empty)
}

#[derive(ctor, Debug, PartialEq)]
pub struct OneFieldStruct {
    value: u32
}

#[test]
fn test_struct_with_field() {
    let ofs = OneFieldStruct::new(300);
    assert_eq!(OneFieldStruct { value: 300 }, ofs)
}

#[derive(ctor, Debug, PartialEq)]
pub struct ManyFieldStruct {
    value1: u32,
    value2: bool
}

#[test]
fn test_struct_with_many_fields() {
    let mfs = ManyFieldStruct::new(400, true);
    assert_eq!(ManyFieldStruct { value1: 400, value2: true }, mfs);
}