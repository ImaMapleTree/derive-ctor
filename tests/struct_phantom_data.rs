use derive_ctor::ctor;
use std::marker::PhantomData;

#[derive(ctor, Debug, PartialEq)]
struct HasPhantomData {
    value: u32,
    _marker: PhantomData<u32>,
}

#[test]
fn test_phantom_data_auto_excluded_as_parameter() {
    let pd = HasPhantomData::new(4);
    assert_eq!(
        HasPhantomData {
            value: 4,
            _marker: PhantomData
        },
        pd
    )
}
