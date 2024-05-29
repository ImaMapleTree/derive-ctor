use derive_ctor::ctor;

#[derive(ctor, Debug, PartialEq)]
struct StructImpl {
    #[ctor(into)]
    provided: String,
    other: bool
}

#[derive(ctor, Debug, PartialEq)]
struct StructManyImpl {
    provided: bool,
    #[ctor(into)]
    one: String,
    #[ctor(into)]
    two: String
}

#[test]
fn test_struct_with_impl_value() {
    let test = StructImpl::new("Foo", false);
    assert_eq!(StructImpl { provided: String::from("Foo"), other: false }, test);
}

#[test]
fn test_struct_with_many_impl() {
    let test = StructManyImpl::new(false, "One", "Two");
    assert_eq!(StructManyImpl { provided: false, one: String::from("One"), two: String::from("Two") }, test)
}