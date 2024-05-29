use derive_ctor::ctor;

#[derive(ctor, Debug, PartialEq)]
enum UnnamedVariantEnum {
    One(#[ctor(default)] i32)
}

#[test]
fn test_unnamed_variant_property() {
    let result = UnnamedVariantEnum::one();
    assert_eq!(UnnamedVariantEnum::One(0), result)
}

#[derive(ctor, Debug, PartialEq)]
enum UnnamedVariantEnumMultipleFields {
    Many(i32, #[ctor(expr(false))] bool, #[ctor(into)] String)
}

#[test]
fn test_unnamed_variant_multiple_properties() {
    let result = UnnamedVariantEnumMultipleFields::many(50, "FooBar");
    assert_eq!(UnnamedVariantEnumMultipleFields::Many(50, false, String::from("FooBar")), result);
}

#[derive(ctor, Debug, PartialEq)]
enum EnumMultipleUnnamedVariants {
    One(i32),
    Two(bool, #[ctor(default)] String),
    Three(#[ctor(cloned)] String)
}

#[test]
fn test_multiple_enum_variants() {
    let one = EnumMultipleUnnamedVariants::one(13);
    assert_eq!(EnumMultipleUnnamedVariants::One(13), one);
    
    let two = EnumMultipleUnnamedVariants::two(false);
    assert_eq!(EnumMultipleUnnamedVariants::Two(false, String::default()), two);
    
    let s = String::from("Test");
    let three = EnumMultipleUnnamedVariants::three(&s);
    assert_eq!(EnumMultipleUnnamedVariants::Three(s), three);
}