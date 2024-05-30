use derive_ctor::ctor;

#[derive(ctor, Debug, PartialEq)]
struct StructDefault {
    provided: String,
    #[ctor(default)]
    generated: u32,
}

#[derive(ctor, Debug, PartialEq)]
struct StructManyDefault {
    provided: String,
    #[ctor(default)]
    generated1: u32,
    #[ctor(default)]
    generated2: String,
}

#[test]
fn test_struct_with_default_field() {
    let test = StructDefault::new(String::from("ABC"));
    assert_eq!(
        StructDefault {
            provided: String::from("ABC"),
            generated: Default::default()
        },
        test
    );
}

#[test]
fn test_struct_with_multiple_default_fields() {
    let test = StructManyDefault::new(String::from("ABC"));
    assert_eq!(
        StructManyDefault {
            provided: String::from("ABC"),
            generated1: Default::default(),
            generated2: Default::default()
        },
        test
    );
}
