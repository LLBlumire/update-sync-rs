use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

#[cfg(feature = "derive")]
pub mod derive {
    pub use update_sync_derive::*;
}

/// Provides a method to syncronise data
///
/// This should be implemented such that if set and last_base differ, set is returned
/// while if they are the same, then new_base is returned.
///
/// This enables a form of change detection syncronisation, where set takes priority
/// over the last_base.
pub trait UpdateSync {
    /// Implementations will generally take the form
    ///
    /// ```.ignore
    /// if last_base != set {
    ///     set
    /// } else {
    ///     new_base
    /// }
    fn update_sync(last_base: Self, new_base: Self, set: Self) -> Self;
}

macro_rules! default_impl_update_sync {
    ($c:ty) => {
        impl UpdateSync for $c {
            fn update_sync(last_base: Self, new_base: Self, set: Self) -> Self {
                if last_base != set {
                    set
                } else {
                    new_base
                }
            }
        }
    };
    [$($c:ty),*] => {
        $(
            default_impl_update_sync!($c);
        )+
    };

}
default_impl_update_sync![u8, u16, u32, u64, u128, usize];
default_impl_update_sync![i8, i16, i32, i64, i128, isize];
default_impl_update_sync![f32, f64];
default_impl_update_sync!(bool);
default_impl_update_sync!(char);

// This is highly subject to change
default_impl_update_sync!(String);
// This is esepcailly dodgy, but will likely remain specialised like this as a specialised implementation, because arbitrary binary data is hopefully less volatile than Vec<T>
default_impl_update_sync!(Vec<u8>);

impl<T: PartialEq> UpdateSync for Option<T> {
    fn update_sync(last_base: Self, new_base: Self, set: Self) -> Self {
        if last_base != set {
            set
        } else {
            new_base
        }
    }
}

macro_rules! tuple_impl_update_sync {
    ($($t:ident : $i:tt),+) => {
        impl<$($t),+> UpdateSync for ($($t,)+)
        where
        $(
            $t: UpdateSync,
        )*
        {
            fn update_sync(last_base: Self, new_base: Self, set: Self) -> Self {
                (
                    $(
                        UpdateSync::update_sync(last_base.$i, new_base.$i, set.$i),
                    )*
                )
            }
        }

    }
}

tuple_impl_update_sync!(T1: 0);
tuple_impl_update_sync!(T1: 0, T2: 1);
tuple_impl_update_sync!(T1: 0, T2: 1, T3: 2);
tuple_impl_update_sync!(T1: 0, T2: 1, T3: 2, T4: 3);
tuple_impl_update_sync!(T1: 0, T2: 1, T3: 2, T4: 3, T5: 4);
tuple_impl_update_sync!(T1: 0, T2: 1, T3: 2, T4: 3, T5: 4, T6: 5);
tuple_impl_update_sync!(T1: 0, T2: 1, T3: 2, T4: 3, T5: 4, T6: 5, T7: 6);
tuple_impl_update_sync!(T1: 0, T2: 1, T3: 2, T4: 3, T5: 4, T6: 5, T7: 6, T8: 7);
tuple_impl_update_sync!(T1: 0, T2: 1, T3: 2, T4: 3, T5: 4, T6: 5, T7: 6, T8: 7, T9: 8);
tuple_impl_update_sync!(T1: 0, T2: 1, T3: 2, T4: 3, T5: 4, T6: 5, T7: 6, T8: 7, T9: 8, T10 : 9);

macro_rules! map_impl_update_sync {
    ($t:tt, $($traits:tt)*) => {
        impl<K, V> UpdateSync for $t<K, V>
        where
            K: $($traits)*,
            V: UpdateSync + PartialEq,
        {
            fn update_sync(last_base: Self, mut new_base: Self, mut set: Self) -> Self {
                let mut new = Self::new();
                // First check for changes to base fields
                for (last_base_key, last_base_value) in last_base.into_iter() {
                    let n = new_base.remove(&last_base_key);
                    let s = match set.remove(&last_base_key) {
                        Some(sv) if sv != last_base_value => sv,
                        _ => match n {
                            None => continue, // If it is removed from new base, remove
                            Some(nv) => nv,
                        },
                    };
                    new.insert(last_base_key, s);
                }
                // Next, grab any new entries from the new base
                for (nk, nv) in new_base.into_iter() {
                    let s = match set.remove(&nk) {
                        None => nv,
                        Some(sv) => sv,
                    };
                    new.insert(nk, s);
                }
                // Finally, bring in any new entries from the set
                for (sk, sv) in set.into_iter() {
                    new.insert(sk, sv);
                }

                new
            }
        }
    };
}
map_impl_update_sync!(BTreeMap, Ord);
map_impl_update_sync!(HashMap, Hash + Eq);
