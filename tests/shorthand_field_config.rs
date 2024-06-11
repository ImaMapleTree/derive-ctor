#[cfg(feature = "shorthand")]
use derive_ctor::ctor;

#[cfg(feature = "shorthand")]
#[derive(ctor, Debug, PartialEq)]
struct ShorthandStruct {
    #[expr(100)]
    value1: u32,
    #[cloned]
    value2: String,
    #[into]
    value3: String,
    #[iter(usize)]
    value4: Vec<usize>,
    #[default]
    value5: Option<String>
}

#[test]
#[cfg(feature = "shorthand")]
fn test_struct_with_shorthand() {
    let string = "Foo".to_string();
    let array = [1];
    let test = ShorthandStruct::new(&string, "Bar", array);
    assert_eq!(ShorthandStruct {
        value1: 100,
        value2: string,
        value3: "Bar".to_string(),
        value4: vec![1],
        value5: None
    }, test);
}