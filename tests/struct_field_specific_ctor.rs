use derive_ctor::ctor;

#[derive(ctor, Debug, PartialEq)]
#[ctor(new, new2)]
struct TargetedGenerationStruct {
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
struct TargetedGenerationStruct2 {
    #[ctor(value(test_method_2()) = [1])]
    arg1: String,
    #[ctor(value(33) = 0)]
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
struct TestStructWithFieldWithMultipleTargets {
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