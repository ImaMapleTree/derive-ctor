use derive_ctor::ctor;

#[derive(ctor, Debug, PartialEq)]
#[ctor(prefix = new)]
enum PrefixEnum {
    Variant1,
    Variant2(i32),
    Variant3 { value: i32 }
}

#[test]
fn test_variants_with_prefix() {
    let p1 = PrefixEnum::new_variant1();
    let p2 = PrefixEnum::new_variant2(123);
    let p3 = PrefixEnum::new_variant3(456);
    assert_eq!(PrefixEnum::Variant1, p1);
    assert_eq!(PrefixEnum::Variant2(123), p2);
    assert_eq!(PrefixEnum::Variant3 { value: 456 }, p3)
}

#[derive(ctor)]
#[ctor(vis = pub(crate))]
enum VisibilityEnum {
    Element
}

#[test]
fn test_visibility_enum() {
    let result = VisibilityEnum::element();
    assert!(matches!(result, VisibilityEnum::Element));
}

#[derive(ctor)]
#[ctor(prefix = new, visibility = pub(crate))]
enum PrefixAndVisibilityEnum {
    Element
}

#[test]
fn test_visibility_and_prefix_enum() {
    let result = PrefixAndVisibilityEnum::new_element();
    assert!(matches!(result, PrefixAndVisibilityEnum::Element))
}