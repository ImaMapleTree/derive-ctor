#[macro_use]
extern crate derive_ctor;

#[derive(ctor, Debug, PartialEq)]
pub struct Empty {}

#[test]
fn test_empty_struct_no_config() {
    let empty = Empty::new();
    assert_eq!(Empty { }, empty)
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(init)]
pub struct Empty2 {}

#[test]
fn test_empty_struct_config_name() {
    let empty = Empty2::init();
    assert_eq!(Empty2 { }, empty)
}

#[derive(ctor, Debug, PartialEq)]
pub struct Unit;

#[test]
fn test_unit_struct() {
    let unit = Unit::new();
    assert_eq!(Unit, unit);
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(pub(crate) new)]
pub struct VisibilityStruct {}

#[test]
fn test_method_visibility() {
    let visibility = VisibilityStruct::new();
    assert_eq!(VisibilityStruct {}, visibility)
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(m1, m2)]
pub struct ManyMethodStruct {}

#[test]
fn test_empty_struct_many_methods() {
    let many1 = ManyMethodStruct::m1();
    assert_eq!(ManyMethodStruct {}, many1);
    let many2 = ManyMethodStruct::m2();
    assert_eq!(ManyMethodStruct {}, many2);
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(pub m1, pub(crate) m2, m3)]
pub struct ManyVisibleMethodStruct {}

#[test]
fn test_empty_struct_many_methods_with_visibility() {
    let m1 = ManyVisibleMethodStruct::m1();
    assert_eq!(ManyVisibleMethodStruct {}, m1);
    let m2 = ManyVisibleMethodStruct::m2();
    assert_eq!(ManyVisibleMethodStruct {}, m2);
    let m3 = ManyVisibleMethodStruct::m3();
    assert_eq!(ManyVisibleMethodStruct {}, m3);
}

#[derive(ctor, Debug, PartialEq)]
pub struct OneFieldStruct {
    value: u32
}

#[test]
fn test_struct_with_field() {
    let ofs = OneFieldStruct::new(300);
    assert_eq!(OneFieldStruct { value: 300 }, ofs)
}

#[derive(ctor, Debug, PartialEq)]
pub struct ManyFieldStruct {
    value1: u32,
    value2: bool
}

#[test]
fn test_struct_with_many_fields() {
    let mfs = ManyFieldStruct::new(400, true);
    assert_eq!(ManyFieldStruct { value1: 400, value2: true }, mfs);
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(init)]
pub struct FieldStructCustomCtor {
    value: u32
}

#[test]
fn test_field_struct_with_custom_ctor_name() {
    let field_struct = FieldStructCustomCtor::init(15);
    assert_eq!(FieldStructCustomCtor { value: 15 }, field_struct);
}

#[derive(ctor, Debug, PartialEq)]
pub struct DefaultValueStruct {
    provided: String,
    #[ctor(default)]
    generated: u32
}

#[test]
fn test_struct_with_default_field() {
    let test = DefaultValueStruct::new(String::from("ABC"));
    assert_eq!(DefaultValueStruct { provided: String::from("ABC"), generated: Default::default() }, test);
}

#[derive(ctor, Debug, PartialEq)]
pub struct ValueValueStruct {
    provided: u32,
    #[ctor(value(10))]
    generated: u32
}

#[test]
fn test_struct_with_value_field() {
    let test = ValueValueStruct::new(100);
    assert_eq!(ValueValueStruct { provided: 100, generated: 10 }, test);
}

fn generation_method() -> Option<Option<u32>> {
    Some(Some(4123))
}

#[derive(ctor, Debug, PartialEq)]
pub struct MethodValueStruct {
    provided: bool,
    #[ctor(method(generation_method))]
    generated: Option<Option<u32>>
}

#[test]
fn test_struct_with_method_field() {
    let test = MethodValueStruct::new(false);
    assert_eq!(MethodValueStruct { provided: false, generated: Some(Some(4123))}, test)
}

#[derive(ctor, Debug, PartialEq)]
pub struct ImplValueStruct {
    #[ctor(impl)]
    provided: String,
    other: bool
}

#[test]
fn test_struct_with_impl_value() {
    let test = ImplValueStruct::new("Foo", false);
    assert_eq!(ImplValueStruct { provided: String::from("Foo"), other: false }, test);
}

#[derive(ctor, Debug, PartialEq)]
pub struct MultipleGenerated {
    provided1: i16,
    provided2: bool,
    #[ctor(value("Foo"))]
    generated1: &'static str,
    #[ctor(default)]
    generated2: u32
}

#[test]
fn test_struct_with_multiple_generated_fields() {
    let multi = MultipleGenerated::new(41, false);
    assert_eq!(MultipleGenerated { provided1: 41, provided2: false, generated1: "Foo", generated2: Default::default() }, multi)
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(new, new2)]
pub struct TargetedGenerationStruct {
    arg1: u32,
    #[ctor(default = [0])]
    arg2: u32
}

#[test]
fn test_struct_with_targeted_generation() {
    let targeted = TargetedGenerationStruct::new(100);
    assert_eq!(TargetedGenerationStruct { arg1: 100, arg2: Default::default() }, targeted);
    let targeted2 = TargetedGenerationStruct::new2(50, 41);
    assert_eq!(TargetedGenerationStruct { arg1: 50, arg2: 41 }, targeted2);
}

fn test_method_2() -> String {
    String::from("FooBar")
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(new, new2)]
pub struct TargetedGenerationStruct2 {
    #[ctor(method(test_method_2) = [1])]
    arg1: String,
    #[ctor(value(33) = [0])]
    arg2: u32
}

#[test]
fn test_struct_with_multiple_targeted_generations() {
    let tgs1 = TargetedGenerationStruct2::new(String::from("one"));
    assert_eq!(TargetedGenerationStruct2 { arg1: String::from("one"), arg2: 33 }, tgs1);

    let tgs2 = TargetedGenerationStruct2::new2(95);
    assert_eq!(TargetedGenerationStruct2 { arg1: String::from("FooBar"), arg2: 95}, tgs2);
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(m1, m2, m3)]
pub struct TestStructWithFieldWithMultipleTargets {
    #[ctor(impl)]
    arg1: String,
    #[ctor(value(5) = [0, 1])]
    arg2: u32
}

#[test]
fn test_struct_multiple_targeted_generations_single_field() {
    let tswfwmt1 = TestStructWithFieldWithMultipleTargets::m1("One");
    assert_eq!(TestStructWithFieldWithMultipleTargets { arg1: String::from("One"), arg2: 5 }, tswfwmt1);

    let tswfwmt1 = TestStructWithFieldWithMultipleTargets::m2("Two");
    assert_eq!(TestStructWithFieldWithMultipleTargets { arg1: String::from("Two"), arg2: 5 }, tswfwmt1);

    let tswfwmt1 = TestStructWithFieldWithMultipleTargets::m3("Three", 77);
    assert_eq!(TestStructWithFieldWithMultipleTargets { arg1: String::from("Three"), arg2: 77 }, tswfwmt1);
}