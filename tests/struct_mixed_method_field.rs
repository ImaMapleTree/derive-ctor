use derive_ctor::ctor;

#[derive(ctor, Debug, PartialEq)]
#[ctor(a)]
struct MixedCtorFieldStruct {
    provided: u32,
    #[ctor(default)]
    generated: bool
}

#[test]
fn test_struct_with_custom_ctor_and_generated_field() {
    let test = MixedCtorFieldStruct::a(100);
    assert_eq!(MixedCtorFieldStruct { provided: 100, generated: false }, test)
}