use std::collections::BTreeMap;

use dodb_core::PageId;

use super::format::PageData;

#[derive(Clone, Debug)]
pub(crate) struct PageCache {
    capacity: usize,
    pages: BTreeMap<PageId, CacheEntry>,
    clock: u64,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    page: PageData,
    last_used: u64,
}

impl PageCache {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            capacity,
            pages: BTreeMap::new(),
            clock: 0,
        }
    }

    pub(crate) fn get(&mut self, page_id: PageId) -> Option<PageData> {
        let clock = self.next_clock();
        let entry = self.pages.get_mut(&page_id)?;
        entry.last_used = clock;
        Some(entry.page.clone())
    }

    pub(crate) fn insert(&mut self, page_id: PageId, page: PageData) {
        if self.capacity == 0 {
            return;
        }
        let clock = self.next_clock();
        self.pages.insert(
            page_id,
            CacheEntry {
                page,
                last_used: clock,
            },
        );
        while self.pages.len() > self.capacity {
            let Some((victim, _)) = self
                .pages
                .iter()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(page_id, entry)| (*page_id, entry.last_used))
            else {
                break;
            };
            self.pages.remove(&victim);
        }
    }

    pub(crate) fn insert_many(&mut self, pages: impl IntoIterator<Item = (PageId, PageData)>) {
        for (page_id, page) in pages {
            self.insert(page_id, page);
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.pages.len()
    }

    fn next_clock(&mut self) -> u64 {
        self.clock = self.clock.wrapping_add(1);
        self.clock
    }
}
