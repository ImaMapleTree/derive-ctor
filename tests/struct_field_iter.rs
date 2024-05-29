use std::collections::HashSet;

use derive_ctor::ctor;

#[derive(ctor, Debug, PartialEq)]
struct StructIter {
    #[ctor(iter(usize))]
    collection: HashSet<usize>
}

#[test]
fn test_struct_with_field_iter() {
    let test = StructIter::new(vec![1, 3, 6, 6]);

    let mut expected_set = HashSet::new();
    expected_set.insert(1);
    expected_set.insert(3);
    expected_set.insert(6);

    assert_eq!(StructIter { collection: expected_set }, test);
}