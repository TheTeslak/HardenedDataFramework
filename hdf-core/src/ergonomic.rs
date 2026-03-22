use crate::hardened::Hardened;
use crate::scheme::{Dmr, Tmr};
use crate::store::{InlineStore, ReplicaStore};

pub type DetectOnly<T, Store> = Hardened<T, Dmr, Store, 2>;
pub type Recoverable<T, Store> = Hardened<T, Tmr, Store, 3>;

pub type DetectOnlyInline<T> = DetectOnly<T, InlineStore<T, 2>>;
pub type RecoverableInline<T> = Recoverable<T, InlineStore<T, 3>>;

pub fn detect_only<T>(initial: T) -> DetectOnlyInline<T>
where
    T: Copy + Eq,
{
    Hardened::new(initial, InlineStore::new(initial))
}

pub fn detect_only_in<T, Store>(initial: T, store: Store) -> DetectOnly<T, Store>
where
    T: Copy + Eq,
    Store: ReplicaStore<T, 2>,
{
    Hardened::new(initial, store)
}

pub fn recoverable<T>(initial: T) -> RecoverableInline<T>
where
    T: Copy + Eq,
{
    Hardened::new(initial, InlineStore::new(initial))
}

pub fn recoverable_in<T, Store>(initial: T, store: Store) -> Recoverable<T, Store>
where
    T: Copy + Eq,
    Store: ReplicaStore<T, 3>,
{
    Hardened::new(initial, store)
}
