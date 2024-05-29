use derive_ctor::ctor;

#[derive(ctor, Debug, PartialEq)]
enum EnumUnitVariant {
    Variant1
}

#[derive(ctor, Debug, PartialEq)]
enum EnumStructVariant {
    Variant { field: i32 }
}

#[derive(ctor, Debug, PartialEq)]
enum EnumNamelessVariant {
    Variant(i32)
}

#[test]
fn test_enum_variants() {
    let unit = EnumUnitVariant::variant1();
    assert_eq!(EnumUnitVariant::Variant1, unit);
    
    let struct_variant = EnumStructVariant::variant(13);
    assert_eq!(EnumStructVariant::Variant { field: 13 }, struct_variant);
    
    let nameless_variant = EnumNamelessVariant::variant(95);
    assert_eq!(EnumNamelessVariant::Variant(95), nameless_variant);
}

#[derive(ctor, Debug, PartialEq)]
enum MultipleVariants {
    Variant1,
    Variant2(i32),
    Variant3 { value: usize }
}

#[test]
fn test_enum_multiple_variants() {
    let v1 = MultipleVariants::variant1();
    assert_eq!(MultipleVariants::Variant1, v1);
    
    let v2 = MultipleVariants::variant2(123);
    assert_eq!(MultipleVariants::Variant2(123), v2);
    
    let v3 = MultipleVariants::variant3(888);
    assert_eq!(MultipleVariants::Variant3 { value: 888 }, v3);
}