#![cfg(test)]
use std::collections::BTreeMap;
use update_sync::UpdateSync;

#[test]
fn maps() {
    let mut base_map = BTreeMap::new();
    base_map.insert(1, 'a');

    let mut user_one = base_map.clone();
    let mut user_two = base_map.clone();

    user_one.insert(2, 'c');
    user_one.insert(3, 'c');

    user_two.insert(2, 'b');

    let mut should_be = BTreeMap::new();
    should_be.insert(1, 'a');
    should_be.insert(2, 'b');
    should_be.insert(3, 'c');

    // users ones changes come in before user two
    let is = UpdateSync::update_sync(base_map, user_one, user_two);
    assert_eq!(is, should_be);
}

#[derive(update_sync::derive::UpdateSync, PartialEq, Debug)]
pub struct UnitStruct;

#[test]
fn unit_struct() {
    let base = UnitStruct;
    let one = UnitStruct;
    let two = UnitStruct;

    let should_be = UnitStruct;

    let is = UpdateSync::update_sync(base, one, two);

    assert_eq!(is, should_be);
}

#[derive(update_sync::derive::UpdateSync, PartialEq, Debug)]
struct WithFields {
    foo: i32,
    bar: char,
    bat: u8,
}

#[test]
fn with_fields() {
    let base = WithFields {
        foo: 1,
        bar: '\0',
        bat: 0,
    };

    let user_one = WithFields {
        bar: 'c',
        bat: 3,
        ..base
    };
    let user_two = WithFields { bar: 'b', ..base };

    let should_be = WithFields {
        foo: 1,
        bar: 'b',
        bat: 3,
    };

    let is = UpdateSync::update_sync(base, user_one, user_two);

    assert_eq!(is, should_be);
}

#[derive(update_sync::derive::UpdateSync, PartialEq, Debug)]
struct WithUnnamedFields(i32, char, u8);

#[test]
fn with_unnamed_fields() {
    let base = WithUnnamedFields(1, '\0', 0);

    let user_one = WithUnnamedFields(1, 'c', 3);
    let user_two = WithUnnamedFields(1, 'b', 0);

    let should_be = WithUnnamedFields(1, 'b', 3);

    let is = UpdateSync::update_sync(base, user_one, user_two);

    assert_eq!(is, should_be);
}
