use dodb_core::{PageId, Revision};

use crate::churn::{self, ChurnCounter};

const LEAF_BYTES_SLACK: usize = 128;
const LEAF_SLOTS_SLACK: usize = 2;
const LEAF_GARBAGE_FLOOR: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BlinkValueRef<'a> {
    Inline(&'a [u8]),
    Overflow { head: PageId, length: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct LeafEntryRef<'a> {
    pub(super) key: &'a [u8],
    pub(super) revision: Revision,
    pub(super) value: Option<BlinkValueRef<'a>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StoredValue {
    Inline,
    Overflow { head: PageId, length: u64 },
}

#[derive(Clone, Copy, Debug)]
enum SlotValue {
    Missing,
    Inline { offset: u32, length: u32 },
    Overflow { head: PageId, length: u64 },
}

#[derive(Clone, Copy, Debug)]
struct LeafSlot {
    revision: Revision,
    key_offset: u32,
    key_length: u32,
    value: SlotValue,
}

/// The entries of one leaf. Keys and inline values live in one byte buffer
/// owned by the leaf; each slot holds offsets into it, so cloning a leaf
/// copies two vectors and never touches a per-entry reference count.
/// Bytes of replaced or removed payloads stay in the buffer as garbage until
/// the leaf is cloned or the garbage outgrows the live bytes.
#[derive(Default)]
pub(super) struct LeafEntries {
    slots: Vec<LeafSlot>,
    bytes: Vec<u8>,
    garbage: usize,
}

impl LeafEntries {
    pub(super) fn with_capacity(entries: usize, payload_bytes: usize) -> Self {
        Self {
            slots: Vec::with_capacity(entries),
            bytes: Vec::with_capacity(payload_bytes),
            garbage: 0,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.slots.len()
    }

    fn slot_ref(&self, slot: &LeafSlot) -> LeafEntryRef<'_> {
        let key_start = slot.key_offset as usize;
        LeafEntryRef {
            key: &self.bytes[key_start..key_start + slot.key_length as usize],
            revision: slot.revision,
            value: match slot.value {
                SlotValue::Missing => None,
                SlotValue::Inline { offset, length } => Some(BlinkValueRef::Inline(
                    &self.bytes[offset as usize..offset as usize + length as usize],
                )),
                SlotValue::Overflow { head, length } => {
                    Some(BlinkValueRef::Overflow { head, length })
                }
            },
        }
    }

    pub(super) fn get(&self, index: usize) -> LeafEntryRef<'_> {
        self.slot_ref(&self.slots[index])
    }

    pub(super) fn key(&self, index: usize) -> &[u8] {
        let slot = &self.slots[index];
        &self.bytes[slot.key_offset as usize..(slot.key_offset + slot.key_length) as usize]
    }

    pub(super) fn first(&self) -> Option<LeafEntryRef<'_>> {
        self.slots.first().map(|slot| self.slot_ref(slot))
    }

    pub(super) fn last(&self) -> Option<LeafEntryRef<'_>> {
        self.slots.last().map(|slot| self.slot_ref(slot))
    }

    pub(super) fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = LeafEntryRef<'_>> + ExactSizeIterator + '_ {
        self.slots.iter().map(|slot| self.slot_ref(slot))
    }

    pub(super) fn range(&self, start: usize, end: usize) -> LeafRange<'_> {
        assert!(start <= end && end <= self.len());
        LeafRange {
            entries: self,
            start,
            end,
        }
    }

    pub(super) fn all(&self) -> LeafRange<'_> {
        self.range(0, self.len())
    }

    pub(super) fn search(&self, key: &[u8]) -> Result<usize, usize> {
        self.slots.binary_search_by(|slot| {
            churn::add(ChurnCounter::LeafKeyComparisons, 1);
            self.bytes[slot.key_offset as usize..(slot.key_offset + slot.key_length) as usize]
                .cmp(key)
        })
    }

    #[cfg(test)]
    pub(super) fn garbage_bytes(&self) -> usize {
        self.garbage
    }

    fn append_bytes(&mut self, bytes: &[u8]) -> u32 {
        let offset = self.bytes.len();
        self.bytes.extend_from_slice(bytes);
        u32::try_from(offset).expect("Blink leaf payload buffer exceeds u32")
    }

    fn stored_slot_value(&mut self, value: Option<BlinkValueRef<'_>>) -> SlotValue {
        match value {
            None => SlotValue::Missing,
            Some(BlinkValueRef::Inline(bytes)) => SlotValue::Inline {
                offset: self.append_bytes(bytes),
                length: u32::try_from(bytes.len()).expect("Blink inline value exceeds u32"),
            },
            Some(BlinkValueRef::Overflow { head, length }) => SlotValue::Overflow { head, length },
        }
    }

    fn new_slot(&mut self, entry: LeafEntryRef<'_>) -> LeafSlot {
        let key_offset = self.append_bytes(entry.key);
        LeafSlot {
            revision: entry.revision,
            key_offset,
            key_length: u32::try_from(entry.key.len()).expect("Blink leaf key exceeds u32"),
            value: self.stored_slot_value(entry.value),
        }
    }

    pub(super) fn push(&mut self, entry: LeafEntryRef<'_>) {
        let slot = self.new_slot(entry);
        self.slots.push(slot);
    }

    pub(super) fn insert(&mut self, index: usize, entry: LeafEntryRef<'_>) {
        let slot = self.new_slot(entry);
        self.slots.insert(index, slot);
    }

    fn stored_value(value: SlotValue) -> Option<StoredValue> {
        match value {
            SlotValue::Missing => None,
            SlotValue::Inline { .. } => Some(StoredValue::Inline),
            SlotValue::Overflow { head, length } => Some(StoredValue::Overflow { head, length }),
        }
    }

    fn inline_length(value: SlotValue) -> usize {
        match value {
            SlotValue::Inline { length, .. } => length as usize,
            _ => 0,
        }
    }

    /// Replaces the revision and value of the entry at `index`, whose key must
    /// equal `key`. A new inline value of the same length is written over the
    /// old bytes; any other change appends and leaves the old bytes as garbage.
    pub(super) fn replace(
        &mut self,
        index: usize,
        key: &[u8],
        revision: Revision,
        value: Option<BlinkValueRef<'_>>,
    ) -> Option<StoredValue> {
        assert_eq!(self.key(index), key, "Blink leaf replace changes the key");
        let old = self.slots[index].value;
        let new_value = match (old, value) {
            (SlotValue::Inline { offset, length }, Some(BlinkValueRef::Inline(bytes)))
                if length as usize == bytes.len() =>
            {
                self.bytes[offset as usize..offset as usize + bytes.len()].copy_from_slice(bytes);
                old
            }
            _ => {
                self.garbage += Self::inline_length(old);
                self.stored_slot_value(value)
            }
        };
        let slot = &mut self.slots[index];
        slot.revision = revision;
        slot.value = new_value;
        self.compact_if_sparse();
        Self::stored_value(old)
    }

    pub(super) fn set_revision(&mut self, index: usize, revision: Revision) {
        self.slots[index].revision = revision;
    }

    #[cfg(test)]
    pub(super) fn remove(&mut self, index: usize) -> Option<StoredValue> {
        let slot = self.slots.remove(index);
        self.garbage += slot.key_length as usize + Self::inline_length(slot.value);
        self.compact_if_sparse();
        Self::stored_value(slot.value)
    }

    pub(super) fn split_off(&mut self, at: usize) -> Self {
        let right = self.range(at, self.len()).to_entries();
        let left = self.range(0, at).to_entries();
        *self = left;
        right
    }

    fn compact_if_sparse(&mut self) {
        if self.garbage > LEAF_GARBAGE_FLOOR && self.garbage * 2 > self.bytes.len() {
            churn::add(ChurnCounter::LeafCompactions, 1);
            *self = self.all().to_entries();
        }
    }

    fn copy_with_slack(&self) -> Self {
        let mut slots = Vec::with_capacity(self.slots.len() + LEAF_SLOTS_SLACK);
        slots.extend_from_slice(&self.slots);
        let mut bytes = Vec::with_capacity(self.bytes.len() + LEAF_BYTES_SLACK);
        bytes.extend_from_slice(&self.bytes);
        Self {
            slots,
            bytes,
            garbage: self.garbage,
        }
    }
}

impl Clone for LeafEntries {
    fn clone(&self) -> Self {
        let cloned = if self.garbage == 0 {
            self.copy_with_slack()
        } else {
            self.all().to_entries()
        };
        if churn::ENABLED {
            churn::add(ChurnCounter::LeafEntriesCopied, cloned.slots.len() as u64);
            churn::add(
                ChurnCounter::LeafSlotBytesCopied,
                (cloned.slots.len() * std::mem::size_of::<LeafSlot>()) as u64,
            );
            churn::add(
                ChurnCounter::LeafPayloadBytesCopied,
                cloned.bytes.len() as u64,
            );
        }
        cloned
    }
}

impl PartialEq for LeafEntries {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl Eq for LeafEntries {}

impl std::fmt::Debug for LeafEntries {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_list().entries(self.iter()).finish()
    }
}

impl<'a> FromIterator<LeafEntryRef<'a>> for LeafEntries {
    fn from_iter<I: IntoIterator<Item = LeafEntryRef<'a>>>(entries: I) -> Self {
        let mut leaf = Self::default();
        for entry in entries {
            leaf.push(entry);
        }
        leaf
    }
}

/// A contiguous run of a leaf's entries, used where the encoder and the split
/// policy look at part of a leaf.
#[derive(Clone, Copy)]
pub(super) struct LeafRange<'a> {
    entries: &'a LeafEntries,
    start: usize,
    end: usize,
}

impl<'a> LeafRange<'a> {
    pub(super) fn len(&self) -> usize {
        self.end - self.start
    }

    pub(super) fn get(&self, index: usize) -> LeafEntryRef<'a> {
        assert!(index < self.len());
        self.entries.get(self.start + index)
    }

    pub(super) fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = LeafEntryRef<'a>> + ExactSizeIterator + 'a {
        let entries = self.entries;
        entries.slots[self.start..self.end]
            .iter()
            .map(move |slot| entries.slot_ref(slot))
    }

    pub(super) fn to_entries(self) -> LeafEntries {
        let payload = self
            .iter()
            .map(|entry| {
                entry.key.len()
                    + match entry.value {
                        Some(BlinkValueRef::Inline(bytes)) => bytes.len(),
                        _ => 0,
                    }
            })
            .sum::<usize>();
        let mut leaf =
            LeafEntries::with_capacity(self.len() + LEAF_SLOTS_SLACK, payload + LEAF_BYTES_SLACK);
        for entry in self.iter() {
            leaf.push(entry);
        }
        leaf
    }
}
