use derive_ctor::ctor;
#[derive(ctor, Debug, PartialEq)]
#[ctor(init)]
struct Empty2 {}

#[test]
fn test_empty_struct_config_name() {
    let empty = Empty2::init();
    assert_eq!(Empty2 {}, empty)
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(pub(crate) new)]
struct VisibilityStruct {}

#[test]
fn test_method_visibility() {
    let visibility = VisibilityStruct::new();
    assert_eq!(VisibilityStruct {}, visibility)
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(m1, m2)]
struct ManyMethodStruct {}

#[test]
fn test_empty_struct_many_methods() {
    let many1 = ManyMethodStruct::m1();
    assert_eq!(ManyMethodStruct {}, many1);
    let many2 = ManyMethodStruct::m2();
    assert_eq!(ManyMethodStruct {}, many2);
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(pub m1, pub(crate) m2, m3)]
struct ManyVisibleMethodStruct {}

#[test]
fn test_empty_struct_many_methods_with_visibility() {
    let m1 = ManyVisibleMethodStruct::m1();
    assert_eq!(ManyVisibleMethodStruct {}, m1);
    let m2 = ManyVisibleMethodStruct::m2();
    assert_eq!(ManyVisibleMethodStruct {}, m2);
    let m3 = ManyVisibleMethodStruct::m3();
    assert_eq!(ManyVisibleMethodStruct {}, m3);
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(init)]
struct FieldStructCustomCtor {
    value: u32,
}

#[test]
fn test_field_struct_with_custom_ctor_name() {
    let field_struct = FieldStructCustomCtor::init(15);
    assert_eq!(FieldStructCustomCtor { value: 15 }, field_struct);
}
