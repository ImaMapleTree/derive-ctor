use derive_ctor::ctor;

#[derive(ctor, Debug, PartialEq)]
enum SpecificVariantMethod {
    #[ctor(const pub new, other)]
    Variant1(i32)
}

#[test]
fn test_variant_with_configured_ctors() {
    const RESULT: SpecificVariantMethod = SpecificVariantMethod::new(15);
    assert_eq!(SpecificVariantMethod::Variant1(15), RESULT);
    
    let result2 = SpecificVariantMethod::other(30);
    assert_eq!(SpecificVariantMethod::Variant1(30), result2);
}

#[derive(ctor, Debug, PartialEq)]
enum EnumNoVariantGeneration {
    Variant1,
    #[ctor(none)] #[allow(dead_code)]
    Variant2
}

#[test]
fn test_enum_with_specified_no_ctor_generation() {
    let variant = EnumNoVariantGeneration::variant1();
    assert_eq!(EnumNoVariantGeneration::Variant1, variant); // you'll have to take my word for it- variant2 does not get generated
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(prefix = new, vis = pub)]
enum VariantConfigOverridesDefaults {
    Variant1,
    #[ctor(variant2)]
    Variant2,
    #[ctor(none)] #[allow(dead_code)]
    Variant3
}

#[test]
fn test_variant_config_overrides_default_values() {
    let variant1 = VariantConfigOverridesDefaults::new_variant1();
    assert_eq!(VariantConfigOverridesDefaults::Variant1, variant1);
    
    let variant2 = VariantConfigOverridesDefaults::variant2();
    assert_eq!(VariantConfigOverridesDefaults::Variant2, variant2);
    
    // variant3 was not generated
}

