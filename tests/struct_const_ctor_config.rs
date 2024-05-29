use derive_ctor::ctor;

#[derive(ctor, Debug, PartialEq)]
#[ctor(const new)]
struct ConstStructV0 {
    value: i32
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(const pub new)]
struct ConstStructV1 {
    value: i32
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(pub const new)]
struct ConstStructV2 {
    value: i32
}

#[derive(ctor, Debug, PartialEq)]
#[ctor(m1, pub(crate) const m2, const m3, m4)]
struct ConstStructMultiple {
    value: i32
}

#[test]
fn test_const_struct_variations() {
    const CONST_STRUCT_V0: ConstStructV0 = ConstStructV0::new(1);
    assert_eq!(ConstStructV0 { value: 1 }, CONST_STRUCT_V0);

    const CONST_STRUCT_V1: ConstStructV1 = ConstStructV1::new(2);
    assert_eq!(ConstStructV1 { value: 2 }, CONST_STRUCT_V1);

    const CONST_STRUCT_V2: ConstStructV2 = ConstStructV2::new(3);
    assert_eq!(ConstStructV2 { value: 3 }, CONST_STRUCT_V2);
}

#[test]
fn test_struct_with_multiple_methods() {
    let v0: ConstStructMultiple = ConstStructMultiple::m1(1);
    assert_eq!(ConstStructMultiple { value: 1 }, v0);

    const V1: ConstStructMultiple = ConstStructMultiple::m2(2);
    assert_eq!(ConstStructMultiple { value: 2 }, V1);

    const V2: ConstStructMultiple = ConstStructMultiple::m3(3);
    assert_eq!(ConstStructMultiple { value: 3 }, V2);
    
    let v3: ConstStructMultiple = ConstStructMultiple::m4(4);
    assert_eq!(ConstStructMultiple { value: 4 }, v3);
}