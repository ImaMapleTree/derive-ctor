use derive_ctor::ctor;

#[derive(ctor, Debug, PartialEq)]
struct StructClone {
    #[ctor(cloned)]
    value: String,
}

#[test]
fn test_struct_clone_field() {
    let value = String::from("Foo");
    let test = StructClone::new(&value);
    assert_eq!(StructClone { value }, test)
}
