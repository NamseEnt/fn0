use std::collections::{BTreeSet, HashSet};

use dodb_core::{Error, PageId, Result, Revision};

use super::format::{INLINE_VALUE_LIMIT, MAX_OVERFLOW_PAGES, MAX_VALUE_SIZE, PageData, ValueRef};
use super::{
    DurableFile, FIRST_DATA_PAGE, InvariantReport, PAGE_SIZE, decode_page_at, read_exact_at,
};

type KeyBounds = (Option<Vec<u8>>, Option<Vec<u8>>);

#[derive(Default)]
struct TreeCheckState {
    reachable: HashSet<PageId>,
    keys: BTreeSet<Vec<u8>>,
    leaves: Vec<PageId>,
    max_revision: Revision,
}

pub(crate) fn check<F: DurableFile>(
    file: &mut F,
    root: PageId,
    free_head: Option<PageId>,
    high_water: PageId,
) -> Result<InvariantReport> {
    let mut checker = Checker::new(file, high_water);
    let mut tree_state = TreeCheckState::default();
    checker.visit_tree(root, None, None, &mut tree_state)?;
    let leftmost = checker.leftmost_leaf(root, &mut HashSet::new())?;
    checker.check_leaf_chain(&tree_state.leaves, leftmost, &tree_state.reachable)?;

    let mut free_pages = HashSet::new();
    let mut next = free_head;
    while let Some(page_id) = next {
        checker.check_page_id(page_id)?;
        if !free_pages.insert(page_id) {
            return Err(Error::corruption("free-page list contains a cycle"));
        }
        if tree_state.reachable.contains(&page_id) {
            return Err(Error::corruption("a page is both reachable and free"));
        }
        let page = checker.read_page(page_id)?;
        match page {
            PageData::Free {
                next: page_next, ..
            } => next = page_next,
            _ => return Err(Error::corruption("free-list entry is not a free page")),
        }
    }

    let mut leaked_pages = Vec::new();
    for value in FIRST_DATA_PAGE..=high_water.get() {
        let page_id = PageId::new(value);
        if !tree_state.reachable.contains(&page_id) && !free_pages.contains(&page_id) {
            leaked_pages.push(page_id);
        }
    }
    Ok(InvariantReport {
        reachable_pages: tree_state.reachable.len(),
        free_pages: free_pages.len(),
        leaked_pages,
        max_revision: tree_state.max_revision,
    })
}

struct Checker<'a, F: DurableFile> {
    file: &'a mut F,
    high_water_page_id: PageId,
}

impl<'a, F: DurableFile> Checker<'a, F> {
    fn new(file: &'a mut F, high_water_page_id: PageId) -> Self {
        Self {
            file,
            high_water_page_id,
        }
    }

    fn check_page_id(&self, page_id: PageId) -> Result<()> {
        if page_id.get() < FIRST_DATA_PAGE || page_id > self.high_water_page_id {
            return Err(Error::corruption(format!(
                "page id {} is outside checker bounds",
                page_id.get()
            )));
        }
        Ok(())
    }

    fn read_page(&mut self, page_id: PageId) -> Result<PageData> {
        self.check_page_id(page_id)?;
        let bytes = read_exact_at(self.file, page_id.get() * PAGE_SIZE as u64, PAGE_SIZE)?;
        PageData::decode(decode_page_at(&bytes, Some(page_id))?)
    }

    fn visit_tree(
        &mut self,
        page_id: PageId,
        lower: Option<Vec<u8>>,
        upper: Option<Vec<u8>>,
        state: &mut TreeCheckState,
    ) -> Result<KeyBounds> {
        if !state.reachable.insert(page_id) {
            return Err(Error::corruption("tree page is reachable more than once"));
        }
        let page = self.read_page(page_id)?;
        match page {
            PageData::Leaf { entries, .. } => {
                let mut min = None;
                let mut max = None;
                for entry in &entries {
                    if lower.as_ref().is_some_and(|bound| entry.key < *bound)
                        || upper.as_ref().is_some_and(|bound| entry.key >= *bound)
                    {
                        return Err(Error::corruption(
                            "leaf key violates parent separator range",
                        ));
                    }
                    if entry.revision == Revision::ZERO {
                        return Err(Error::corruption("leaf entry has zero revision"));
                    }
                    if !state.keys.insert(entry.key.clone()) {
                        return Err(Error::corruption("duplicate leaf key"));
                    }
                    state.max_revision = state.max_revision.max(entry.revision);
                    if let Some(value) = &entry.value {
                        match value {
                            ValueRef::Inline(value) => {
                                if value.len() > INLINE_VALUE_LIMIT {
                                    return Err(Error::corruption(
                                        "inline value exceeds inline limit",
                                    ));
                                }
                            }
                            ValueRef::Overflow { head, length } => {
                                self.check_overflow(*head, *length, &mut state.reachable)?;
                            }
                        }
                    }
                    min.get_or_insert_with(|| entry.key.clone());
                    max = Some(entry.key.clone());
                }
                state.leaves.push(page_id);
                Ok((min, max))
            }
            PageData::Internal {
                leftmost_child,
                entries,
                ..
            } => {
                if entries.is_empty() {
                    return Err(Error::corruption("internal page has no separators"));
                }
                let mut child_bounds = Vec::with_capacity(entries.len() + 1);
                child_bounds.push((lower.clone(), Some(entries[0].key.clone())));
                for pair in entries.windows(2) {
                    child_bounds.push((Some(pair[0].key.clone()), Some(pair[1].key.clone())));
                }
                let last_separator = entries
                    .last()
                    .map(|entry| entry.key.clone())
                    .ok_or_else(|| Error::corruption("internal page has no last separator"))?;
                child_bounds.push((Some(last_separator), upper));
                let mut children = Vec::with_capacity(entries.len() + 1);
                children.push(leftmost_child);
                children.extend(entries.iter().map(|entry| entry.right_child));
                let mut bounds = Vec::with_capacity(children.len());
                for (child, (child_lower, child_upper)) in children.into_iter().zip(child_bounds) {
                    bounds.push(self.visit_tree(child, child_lower, child_upper, state)?);
                }
                for (index, entry) in entries.iter().enumerate() {
                    if let Some(left_max) = &bounds[index].1
                        && left_max >= &entry.key
                    {
                        return Err(Error::corruption("left child violates internal separator"));
                    }
                    if let Some(right_min) = &bounds[index + 1].0
                        && right_min < &entry.key
                    {
                        return Err(Error::corruption("right child violates internal separator"));
                    }
                }
                let min = bounds.iter().find_map(|bound| bound.0.clone());
                let max = bounds.iter().rev().find_map(|bound| bound.1.clone());
                Ok((min, max))
            }
            PageData::Overflow { .. } | PageData::Free { .. } => {
                Err(Error::corruption("tree reaches a non-tree page"))
            }
        }
    }

    fn check_overflow(
        &mut self,
        head: PageId,
        length: u64,
        reachable: &mut HashSet<PageId>,
    ) -> Result<()> {
        if length == 0 || length > MAX_VALUE_SIZE as u64 {
            return Err(Error::corruption("overflow value length is invalid"));
        }
        let mut page_id = Some(head);
        let mut visited = HashSet::new();
        let mut total = 0u64;
        let mut count = 0usize;
        while let Some(current) = page_id {
            if count >= MAX_OVERFLOW_PAGES || !visited.insert(current) {
                return Err(Error::corruption("overflow chain is cyclic or too long"));
            }
            if !reachable.insert(current) {
                return Err(Error::corruption(
                    "overflow page is referenced more than once",
                ));
            }
            let page = self.read_page(current)?;
            let PageData::Overflow {
                next,
                total_length,
                chunk,
                ..
            } = page
            else {
                return Err(Error::corruption(
                    "overflow chain reaches a non-overflow page",
                ));
            };
            if total_length != length {
                return Err(Error::corruption(
                    "overflow page length disagrees with leaf",
                ));
            }
            total = total
                .checked_add(chunk.len() as u64)
                .ok_or_else(|| Error::corruption("overflow content length overflows"))?;
            page_id = next;
            count += 1;
        }
        if total != length {
            return Err(Error::corruption(
                "overflow content length does not match leaf",
            ));
        }
        Ok(())
    }

    fn leftmost_leaf(&mut self, root: PageId, visited: &mut HashSet<PageId>) -> Result<PageId> {
        if !visited.insert(root) {
            return Err(Error::corruption(
                "tree contains a cycle while finding leftmost leaf",
            ));
        }
        match self.read_page(root)? {
            PageData::Leaf { .. } => Ok(root),
            PageData::Internal { leftmost_child, .. } => {
                self.leftmost_leaf(leftmost_child, visited)
            }
            _ => Err(Error::corruption("tree reaches a non-tree page")),
        }
    }

    fn check_leaf_chain(
        &mut self,
        tree_leaves: &[PageId],
        leftmost: PageId,
        reachable: &HashSet<PageId>,
    ) -> Result<()> {
        let mut chain = Vec::new();
        let mut current = Some(leftmost);
        let mut visited = HashSet::new();
        let mut previous_key = None;
        while let Some(page_id) = current {
            if !visited.insert(page_id) {
                return Err(Error::corruption("leaf chain contains a cycle"));
            }
            if !reachable.contains(&page_id) {
                return Err(Error::corruption("leaf chain points outside the tree"));
            }
            let page = self.read_page(page_id)?;
            let PageData::Leaf {
                entries, next_leaf, ..
            } = page
            else {
                return Err(Error::corruption("leaf chain points to a non-leaf page"));
            };
            for entry in entries {
                if previous_key
                    .as_ref()
                    .is_some_and(|previous| *previous >= entry.key)
                {
                    return Err(Error::corruption("leaf chain ordering is invalid"));
                }
                previous_key = Some(entry.key);
            }
            chain.push(page_id);
            current = next_leaf;
        }
        if chain != tree_leaves {
            return Err(Error::corruption(
                "leaf chain is not connected in tree order",
            ));
        }
        Ok(())
    }
}
