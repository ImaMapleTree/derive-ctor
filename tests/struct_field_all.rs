use derive_ctor::ctor;

#[derive(ctor, Debug, PartialEq)]
struct MixedStruct {
    provided1: i16,
    provided2: bool,
    #[ctor(impl)]
    provided3: String,
    #[ctor(value("Foo"))]
    generated1: &'static str,
    #[ctor(default)]
    generated2: u32
}

#[test]
fn test_struct_with_multiple_generated_fields() {
    let multi = MixedStruct::new(41, false, "Test");
    assert_eq!(MixedStruct { provided1: 41, provided2: false, provided3: String::from("Test"), generated1: "Foo", generated2: Default::default() }, multi)
}