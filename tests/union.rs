use derive_ctor::ctor;

#[derive(ctor)]
union UnionOneField {
    value: i32
}

#[derive(ctor)]
union UnionManyField {
    int: i32,
    float: f32,
    uint: u32
}

#[test]
fn test_union_one_field() {
    let uof = UnionOneField::value(-5);
    unsafe { assert_eq!(uof.value, -5); }
}

#[test]
fn test_union_many_fields() {
    let v1 = UnionManyField::int(-10);
    unsafe { assert_eq!(v1.int, -10); }

    let v2 = UnionManyField::float(31.231);
    unsafe { assert_eq!(v2.float, 31.231); }

    let v3 = UnionManyField::uint(231);
    unsafe { assert_eq!(v3.uint, 231) }
}

#[derive(ctor)]
#[ctor(prefix = new, vis = pub(crate))]
union UnionPrefix {
    int: i32,
    float: f32
}

#[test]
fn test_union_ctor_defaults() {
    let v1 = UnionPrefix::new_float(1.1);
    unsafe { assert_eq!(v1.float, 1.1); }

    let v2 = UnionPrefix::new_int(5);
    unsafe { assert_eq!(v2.int, 5) }
}

#[derive(ctor)]
#[ctor(prefix = new)]
union UnionOverride {
    #[ctor(new)]
    int: i32,
    float: f32,
    #[ctor(const new_other)]
    other: u32
}

#[test]
fn test_union_override() {
    let v1 = UnionOverride::new(18);
    unsafe { assert_eq!(v1.int, 18); }

    let v2 = UnionOverride::new_float(91.7);
    unsafe { assert_eq!(v2.float, 91.7); }

    let v3 = UnionOverride::new_other(181);
    unsafe { assert_eq!(v3.other, 181); }
}

#[derive(ctor)]
union UnionDefault {
    #[ctor(default)]
    int: i32
}