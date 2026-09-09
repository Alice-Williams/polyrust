//! Subobject copies preserve snapshots; unknown writes poison unseen cells too.
use super::{Key, Origin, Root, State, merge};

impl<'a> State<'a> {
    pub(in crate::ownership::numeric_flow) fn write_key(&mut self, destination: &Key) {
        let before = self.cells.clone();
        self.kill(destination.root());
        if matches!(
            destination.root().shape(),
            crate::ownership::paths::Shape::Elements { .. }
        ) {
            // An unseen symbolic selection may alias any written element.
            // Exact reaching writes below override this conservative fallback.
            self.fallback_origin(destination.root(), Origin::Incomplete);
        }
        for (key, old) in before {
            if key.root() == destination.root()
                && let Some(value) = self.cells.get_mut(&key)
            {
                if !key.overlaps(destination) {
                    value.domain = old.domain;
                } else if key != *destination {
                    // A different observation may now see the new write's losses.
                    merge(&mut value.losses, &[Origin::Incomplete]);
                }
            }
        }
    }
    pub(in crate::ownership::numeric_flow) fn fallback_origin(
        &mut self,
        root: &Root,
        origin: Origin<'a>,
    ) {
        merge(
            self.fallback.entry(root.clone()).or_default(),
            std::slice::from_ref(&origin),
        );
    }
    pub(in crate::ownership::numeric_flow) fn fresh(&mut self, root: &Root) {
        self.kill(root);
        self.cells.retain(|key, _| key.root() != root);
        self.fallback.remove(root);
    }
    pub(in crate::ownership::numeric_flow) fn poison(&mut self, root: &Root, origin: Origin<'a>) {
        self.kill(root);
        merge(
            self.fallback.entry(root.clone()).or_default(),
            std::slice::from_ref(&origin),
        );
        for (key, value) in &mut self.cells {
            if key.root() == root {
                merge(&mut value.losses, std::slice::from_ref(&origin));
            }
        }
    }
    pub(in crate::ownership::numeric_flow) fn copy(&mut self, before: &Self, from: &Key, to: &Key) {
        self.cells.retain(|key, _| key.rebase(to, to).is_none());
        if to.whole_root() {
            self.fallback.remove(to.root());
        }
        if let Some(origins) = before.fallback.get(from.root()) {
            merge(self.fallback.entry(to.root().clone()).or_default(), origins);
        }
        for (key, number) in &before.cells {
            if let Some(key) = key.rebase(from, to) {
                self.set(key, number.clone());
            }
        }
    }
}
