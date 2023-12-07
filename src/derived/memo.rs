use std::sync::Arc;

use arc_swap::{ArcSwap, Guard};
use crossbeam_utils::atomic::AtomicCell;

use crate::{
    hash::FxDashMap, runtime::local_state::QueryRevisions, DatabaseKeyIndex, Event, EventKind,
    Revision, Runtime,
};

use super::DerivedKeyIndex;

pub(super) struct MemoMap<V> {
    map: FxDashMap<DerivedKeyIndex, ArcSwap<Memo<V>>>,
}

impl<V> Default for MemoMap<V> {
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}

impl<V> MemoMap<V> {
    /// Inserts the memo for the given key; (atomically) overwrites any previously existing memo.-
    pub(super) fn insert(&self, key: DerivedKeyIndex, memo: Memo<V>) {
        self.map.insert(key, ArcSwap::from(Arc::new(memo)));
    }

    /// Removes any existing memo for the given key.
    pub(super) fn remove(&self, key: DerivedKeyIndex) {
        self.map.remove(&key);
    }

    /// Loads the current memo for `key_index`. This does not hold any sort of
    /// lock on the `memo_map` once it returns, so this memo could immediately
    /// become outdated if other threads store into the `memo_map`.
    pub(super) fn get(&self, key: DerivedKeyIndex) -> Option<Guard<Arc<Memo<V>>>> {
        self.map.get(&key).map(|v| v.load())
    }

    /// Iterates over the entries in the map. This holds a read lock while iteration continues.
    pub(super) fn iter(&self) -> impl Iterator<Item = (DerivedKeyIndex, Arc<Memo<V>>)> + '_ {
        self.map
            .iter()
            .map(move |r| (*r.key(), r.value().load_full()))
    }

    /// Clears the memo of all entries.
    pub(super) fn clear(&self) {
        self.map.clear()
    }
}

#[derive(Debug)]
pub(super) struct Memo<V> {
    /// The result of the query, if we decide to memoize it.
    pub(super) value: Option<V>,

    /// Last revision when this memo was verified; this begins
    /// as the current revision.
    pub(super) verified_at: AtomicCell<Revision>,

    /// Revision information
    pub(super) revisions: QueryRevisions,
}

impl<V> Memo<V> {
    pub(super) fn new(value: Option<V>, revision_now: Revision, revisions: QueryRevisions) -> Self {
        Memo {
            value,
            verified_at: AtomicCell::new(revision_now),
            revisions,
        }
    }
    /// True if this memo is known not to have changed based on its durability.
    pub(super) fn check_durability(&self, runtime: &Runtime) -> bool {
        let last_changed = runtime.last_changed_revision(self.revisions.durability);
        let verified_at = self.verified_at.load();
        log::debug!(
            "check_durability(last_changed={:?} <= verified_at={:?}) = {:?}",
            last_changed,
            self.verified_at,
            last_changed <= verified_at,
        );
        last_changed <= verified_at
    }

    /// Mark memo as having been verified in the `revision_now`, which should
    /// be the current revision.
    pub(super) fn mark_as_verified(
        &self,
        db: &dyn crate::Database,
        runtime: &crate::Runtime,
        database_key_index: DatabaseKeyIndex,
    ) {
        db.salsa_event(Event {
            runtime_id: runtime.id(),
            kind: EventKind::DidValidateMemoizedValue {
                database_key: database_key_index,
            },
        });

        self.verified_at.store(runtime.current_revision());
    }
}
