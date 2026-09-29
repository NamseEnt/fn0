/// Rows out of a part in sort order, one bounded page at a time.
///
/// Merge reads a whole group into one `Vec<Row>` before it writes anything, and
/// [`docs/MEMORY_ATTRIBUTION.md`] measured what that costs: at the settle peak
/// merge held **829 of 847 live megabytes** across **7.2 million live
/// allocations**, which is the whole reason the budget gate is red at 2 GiB.
/// The rows are already sorted inside each part, so holding them all is not
/// buying the order — it is only paying for it.
///
/// A page is a run of row groups, sized from the part's own average row width,
/// so a part of wide rows pages more finely than one of narrow rows for the
/// same budget. Row groups are the part's indivisible unit, so one is always
/// read even when the budget says otherwise.
pub struct PartRowStream {
    reader: Arc<PartReader>,
    next_group: u32,
    group_count: u32,
    groups_per_page: u32,
    page: std::vec::IntoIter<Row>,
}

impl PartRowStream {
    pub fn new(reader: Arc<PartReader>, page_bytes: u64) -> Self {
        let meta = reader.meta();
        let group_count = meta.row_group_count;
        let rows_per_group = (meta.row_count / u64::from(group_count.max(1))).max(1);
        let bytes_per_row = meta
            .materialized_bytes
            .checked_div(meta.row_count.max(1))
            .unwrap_or(0)
            .max(1);
        let bytes_per_group = rows_per_group.saturating_mul(bytes_per_row).max(1);
        let groups_per_page = (page_bytes / bytes_per_group).clamp(1, u64::from(group_count.max(1)));
        Self {
            reader,
            next_group: 0,
            group_count,
            groups_per_page: groups_per_page as u32,
            page: Vec::new().into_iter(),
        }
    }

    /// The next row, or `None` once the part is exhausted.
    ///
    /// Not an [`Iterator`], because a page can fail to read and the caller has
    /// to see that as an error rather than as the end of the part.
    fn next_row(&mut self) -> Result<Option<Row>, String> {
        loop {
            if let Some(row) = self.page.next() {
                return Ok(Some(row));
            }
            if self.next_group >= self.group_count {
                return Ok(None);
            }
            let end = self
                .next_group
                .saturating_add(self.groups_per_page)
                .min(self.group_count);
            let rows = self
                .reader
                .read_rows_in_row_groups(self.next_group..end, None)?;
            self.next_group = end;
            self.page = rows.into_iter();
        }
    }
}

/// One row and the stream it came from, ordered so a max-heap yields the
/// smallest sort key.
struct Head {
    row: Row,
    stream: usize,
}

impl PartialEq for Head {
    fn eq(&self, other: &Self) -> bool {
        self.row.sort_key() == other.row.sort_key()
    }
}

impl Eq for Head {}

impl PartialOrd for Head {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Head {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reversed: `BinaryHeap` is a max-heap and a merge wants the minimum.
        other.row.sort_key().cmp(&self.row.sort_key())
    }
}

/// A k-way merge over several parts, yielding every row once, in sort order.
///
/// This is what makes streaming a merge possible at all. Each part is already
/// sorted by `(tenant, timestamp_ns, labels, line, metadata)` — the same key
/// `sort_rows` uses — so merging their heads produces the sorted order without
/// a sort, and therefore without holding the rows a sort would need.
///
/// **Duplicates are dropped here**, as `sort_rows` does, and for the same
/// reason: at-least-once recovery can put the same record in two parts, and the
/// first merge that sees both is where the pair collapses. Equal keys are
/// adjacent in merged order, so this is done by looking at the heap's next head
/// rather than by remembering the last row, which would cost a clone per row.
pub struct MergedRows {
    streams: Vec<PartRowStream>,
    heap: std::collections::BinaryHeap<Head>,
    primed: bool,
}

impl MergedRows {
    pub fn new(readers: &[Arc<PartReader>], page_bytes: u64) -> Self {
        Self {
            streams: readers
                .iter()
                .map(|reader| PartRowStream::new(reader.clone(), page_bytes))
                .collect(),
            heap: std::collections::BinaryHeap::new(),
            primed: false,
        }
    }

    fn push_from(&mut self, stream: usize) -> Result<(), String> {
        if let Some(row) = self.streams[stream].next_row()? {
            self.heap.push(Head { row, stream });
        }
        Ok(())
    }

    fn pop(&mut self) -> Result<Option<Row>, String> {
        if !self.primed {
            for stream in 0..self.streams.len() {
                self.push_from(stream)?;
            }
            self.primed = true;
        }
        let Some(head) = self.heap.pop() else {
            return Ok(None);
        };
        self.push_from(head.stream)?;
        Ok(Some(head.row))
    }

    pub fn next_row(&mut self) -> Result<Option<Row>, String> {
        let Some(row) = self.pop()? else {
            return Ok(None);
        };
        while self
            .heap
            .peek()
            .is_some_and(|head| head.row.sort_key() == row.sort_key())
        {
            self.pop()?;
        }
        Ok(Some(row))
    }
}
