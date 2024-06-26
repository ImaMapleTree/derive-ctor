use derive_ctor::ctor;

#[derive(Debug, PartialEq)]
struct NoDefault {}

#[derive(ctor, Debug, PartialEq)]
#[ctor(default)]
struct DefaultCtorStruct {
    #[ctor(expr(NoDefault {}))]
    name: NoDefault,
    #[ctor(default)]
    value: i32,
}

#[test]
fn test_struct_with_default_ctor() {
    let result = Default::default();
    assert_eq!(
        DefaultCtorStruct {
            name: NoDefault {},
            value: 0
        },
        result
    );
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(pub new, default)]
struct TestDefaultCtorWithTargetedFieldConfig {
    #[ctor(expr(String::from("Default")) = 1)]
    name: String,
    #[ctor(expr(404) = 1)]
    value: u32,
}

#[test]
fn test_struct_with_targeted_field_default_ctor() {
    let non_default = TestDefaultCtorWithTargetedFieldConfig::new(String::from("Foo"), 505);
    assert_eq!(
        TestDefaultCtorWithTargetedFieldConfig {
            name: String::from("Foo"),
            value: 505
        },
        non_default
    );

    let default = Default::default();
    assert_eq!(
        TestDefaultCtorWithTargetedFieldConfig {
            name: String::from("Default"),
            value: 404
        },
        default
    );
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(default(all))]
struct ImplementDefaultAllMembers {
    name: String,
    value: u32,
    #[ctor(expr(NoDefault {}))]
    no_default: NoDefault,
    #[ctor(into)]
    provided: String,
    #[ctor(expr!(adjusted - 10))]
    adjusted: i32
}

#[test]
fn test_struct_implement_default_all_members() {
    let result = Default::default();
    assert_eq!(
        ImplementDefaultAllMembers {
            name: Default::default(),
            value: Default::default(),
            no_default: NoDefault {},
            provided: Default::default(),
            adjusted: 0,
        },
        result
    );
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(with_defaults(default), new(into))]
struct CtorNestedProperties {
    name: String,
    #[ctor(expr!(value + 1))]
    value: u32
}

#[test]
fn test_struct_with_nested_properties() {
    let test1 = CtorNestedProperties::with_defaults();
    assert_eq!(CtorNestedProperties { name: String::default(), value: 0 }, test1);
    
    let test2 = CtorNestedProperties::new("FooBar", 30);
    assert_eq!(CtorNestedProperties { name: String::from("FooBar"), value: 31 },  test2)
    
}