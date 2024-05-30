#![no_std]

extern crate alloc;

use alloc::string::String;
use derive_ctor::ctor;

#[derive(ctor, Debug, PartialEq)]
struct MixedStruct {
    provided1: i16,
    provided2: bool,
    #[ctor(into)]
    provided3: String,
    #[ctor(cloned)]
    provided4: String,
    #[ctor(expr!(partial1 + 100))]
    partial1: u32,
    #[ctor(expr(i32 -> partial2 < 0))]
    partial2: bool,
    #[ctor(expr("Foo"))]
    generated1: &'static str,
    #[ctor(default)]
    generated2: u32,
}

#[test]
fn test_struct_with_multiple_generated_fields() {
    let provided4 = String::from("Bar");

    let multi = MixedStruct::new(41, false, "Test", &provided4, 90, -1238);
    assert_eq!(
        MixedStruct {
            provided1: 41,
            provided2: false,
            provided3: String::from("Test"),
            provided4,
            partial1: 190,
            partial2: true,
            generated1: "Foo",
            generated2: Default::default()
        },
        multi
    )
}
