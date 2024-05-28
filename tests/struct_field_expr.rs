use derive_ctor::ctor;

#[derive(ctor, Debug, PartialEq)]
struct StructExpr {
    provided: u32,
    #[ctor(expr(10))]
    generated: u32
}

#[derive(ctor, Debug, PartialEq)]
struct StructManyExpr {
    provided: u32,
    #[ctor(expr(11))]
    generated1: u32,
    #[ctor(expr(false))]
    generated2: bool
}

#[derive(ctor, Debug, PartialEq)]
struct StructComplexExpr {
    provided: u32,
    #[ctor(expr(String::from("Foo")))]
    generated: String
}

#[derive(ctor, Debug, PartialEq)]
struct StructReliantExpr {
    provided: u32,
    #[ctor(expr(provided.to_string()))]
    generated: String
}

#[test]
fn test_struct_expr_field() {
    let test = StructExpr::new(100);
    assert_eq!(StructExpr { provided: 100, generated: 10 }, test);
}

#[test]
fn test_struct_many_expr_fields() {
    let test = StructManyExpr::new(101);
    assert_eq!(StructManyExpr { provided: 101, generated1: 11, generated2: false }, test);
}

#[test]
fn test_struct_complex_expr_field() {
    let test = StructComplexExpr::new(102);
    assert_eq!(StructComplexExpr { provided: 102, generated: String::from("Foo") }, test);
}

#[test]
fn test_struct_reliant_expr_field() {
    let test = StructReliantExpr::new(103);
    assert_eq!(StructReliantExpr { provided: 103, generated: 103.to_string() }, test);
}