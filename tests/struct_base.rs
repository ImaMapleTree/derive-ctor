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

#[derive(ctor, Debug, PartialEq)]
pub struct GenericStruct<T> {
    item: T
}

#[derive(ctor, Debug, PartialEq)]
pub struct WhereStruct<T>
where T: Into<String> {
    item: T
}

#[derive(ctor, Debug, PartialEq)]
pub struct StructWithClosure {
    closure: fn(usize) -> bool
}

#[derive(ctor, Debug, PartialEq)]
pub struct StructWithClosureGeneric<F>
where F: Fn(usize) -> bool {
    closure: F
}

#[test]
fn test_generic_structs() {
    let generic1: GenericStruct<usize> = GenericStruct::new(400);
    assert_eq!(GenericStruct { item: 400 as usize }, generic1);

    let generic2: WhereStruct<&'static str> = WhereStruct::new("FooBar");
    assert_eq!(WhereStruct { item: "FooBar" }, generic2);
}

#[test]
fn test_closure_structs() {
    let closure1 = StructWithClosure::new(|us| us == 439);
    assert!((closure1.closure)(439));

    let target = 440;
    let closure2 = StructWithClosureGeneric::new(|us| us == target);
    assert!((closure2.closure)(440))
}