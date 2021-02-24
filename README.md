# Update-Sync

Update sync is a multi-editor syncronisation strategy designed for use in a client server model.

```rust
pub trait UpdateSync {
    fn update_sync(last_base: Self, new_base: Self, set: Self) -> Self;
}
```

`last_base` is the last state the client was aware of
`new_base` is the state the server is currently aware of
`set` is the state the client wishes to change to

This function returns the new state of the server, to respond to the client.

In general, the strategy applied is as follows:

```rust
if last_base != set {
    set
} else {
    new_base
}
```

This is applied to the following types `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128`, `isize`, `f32`, `f64`, `bool`, `char`, `String`, `Vec<u8>`, `Option<T: PartialEq>`.

Tuples, `HashMap`, and `BTreeMap` will update each index or keyed value independetly of the others.

# Derive

If you enable the feature `derive`, then you will be able to derive this behaviour.

Structs will be updated such that each field is independently updated, as with tuples, `Hashmap`s, and `BTreeMap`s.

Enums are updated such that if the variant stays the same, they are updated like structs. If the variant changes, the `set` variant will overwrite the base variant.

# Why is there no implementation for <the type I need to have this>

I probably missed it, file an issue and I'll fix it.

Unless it's a vector or similar, in that case you'll need operational transformations, which I'd love to implement for this but I'm not even sure what the best strategy to do so would be. If someone pull requests a sensible implementation I'll approve it.

# Can I see a Demo of how this is supposed to be used?

Sure, check out [`demo.rs`](update-sync_test/src/demo.rs)