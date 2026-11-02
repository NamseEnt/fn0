use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use dodb_core::{Error, Lsn, PageId, Result};

use super::resource::{
    BlinkResourceManager, Reservation, ReservationClass, ResourceCategory, ResourceError,
};
use super::version_chain::{DeltaChainRef, DeltaChainStore};

pub const VERSION_INDEX_NODE_BYTES: usize = 4096;
const NODE_HEADER_BYTES: usize = 112;
const NODE_CHECKSUM_OFFSET: usize = VERSION_INDEX_NODE_BYTES - 32;
const NODE_PAYLOAD_BYTES: usize = NODE_CHECKSUM_OFFSET - NODE_HEADER_BYTES;
const CHILD_REFERENCE_BYTES: usize = 48;
const LEAF_DESCRIPTOR_BYTES: usize = 240;
const NODE_CACHE_RESERVATION_BYTES: u64 = 8192;
const ROOT_PIN_RESERVATION_BYTES: u64 = 256;
const MAX_RADIX_LEVEL: u8 = 15;
const INSERT_RESERVATION_BYTES: u64 = 16 * NODE_CACHE_RESERVATION_BYTES;
const NODE_MAGIC: [u8; 8] = *b"DDBVIDX1";
const NODE_FORMAT_VERSION: u16 = 2;
const NODE_KIND_DURABLE: u16 = 1;
const NODE_KIND_SPILL: u16 = 2;
const NODE_FILE_PREFIX: &str = "version-index-";
const NODE_FILE_SUFFIX: &str = ".node";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StoreIdentity {
    pub database_uuid: [u8; 16],
    pub wal_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeRef {
    pub file_number: u64,
    pub offset: u64,
    pub checksum: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VersionIndexRoot {
    pub generation: u64,
    node: Option<NodeRef>,
}

pub struct VersionIndexRootPin {
    root: VersionIndexRoot,
    _reservation: Reservation,
}

impl VersionIndexRootPin {
    pub fn root(&self) -> VersionIndexRoot {
        self.root
    }
}

impl VersionIndexRoot {
    pub fn empty() -> Self {
        Self {
            generation: 0,
            node: None,
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn node_reference(&self) -> Option<NodeRef> {
        self.node
    }

    pub fn from_node_reference(generation: u64, node: NodeRef) -> Result<Self> {
        if generation == 0 || node.file_number == 0 || node.offset != 0 {
            return Err(Error::corruption("version-index root reference is invalid"));
        }
        Ok(Self {
            generation,
            node: Some(node),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VersionToken {
    Bootstrap {
        page_id: PageId,
    },
    Wal {
        source_file_number: u64,
        record_lsn: Lsn,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum SourceFileKind {
    CheckpointPack = 1,
    WalSegment = 2,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SourceIdentity {
    pub kind: SourceFileKind,
    pub file_number: u64,
    pub header_identity: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceReachability {
    Exact(SourceIdentity),
    AllWalSegments,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WalFrameLocator {
    pub source: SourceIdentity,
    pub frame_offset: u64,
    pub frame_length: u32,
    pub header_crc32c: u32,
    pub payload_offset: u64,
    pub payload_length: u32,
    pub payload_crc32c: u32,
    pub committed_prefix_length: u64,
    pub commit_frame_offset: u64,
    pub commit_lsn: Lsn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BaseFrameLocator {
    pub source: SourceIdentity,
    pub frame_offset: u64,
    pub frame_length: u32,
    pub header_crc32c: u32,
    pub image_lsn: Lsn,
    pub image_crc32c: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VersionLocator {
    CheckpointImage {
        source: SourceIdentity,
        offset: u64,
        source_length: u64,
        image_lsn: Lsn,
        image_crc32c: u32,
    },
    WalImage {
        frame: WalFrameLocator,
    },
    WalDelta {
        frame: WalFrameLocator,
        chain: DeltaChainRef,
        depth: u8,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VersionDescriptor {
    pub page_id: PageId,
    pub token: VersionToken,
    pub image_lsn: Lsn,
    pub image_crc32c: u32,
    pub locator: VersionLocator,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NodeFileKind {
    Durable,
    Spill,
}

impl NodeFileKind {
    fn decode(value: u16) -> Result<Self> {
        match value {
            NODE_KIND_DURABLE => Ok(Self::Durable),
            NODE_KIND_SPILL => Ok(Self::Spill),
            _ => Err(Error::corruption("version-index node kind is unknown")),
        }
    }

    fn encode(self) -> u16 {
        match self {
            Self::Durable => NODE_KIND_DURABLE,
            Self::Spill => NODE_KIND_SPILL,
        }
    }
}

#[derive(Clone)]
enum NodeContent {
    Internal(Box<[Option<NodeRef>; 16]>),
    Leaf(Box<[Option<VersionDescriptor>; 16]>),
}

#[derive(Clone)]
struct IndexNode {
    level: u8,
    generation: u64,
    bitmap: u16,
    kind: NodeFileKind,
    content: NodeContent,
}

impl IndexNode {
    fn empty(level: u8, generation: u64, kind: NodeFileKind) -> Self {
        let content = if level == 0 {
            NodeContent::Leaf(Box::new(std::array::from_fn(|_| None)))
        } else {
            NodeContent::Internal(Box::new(std::array::from_fn(|_| None)))
        };
        Self {
            level,
            generation,
            bitmap: 0,
            kind,
            content,
        }
    }
}

struct CachedNode {
    node: IndexNode,
    _reservation: Reservation,
}

struct NodeCache {
    nodes: HashMap<NodeRef, Arc<CachedNode>>,
    lru: VecDeque<NodeRef>,
}

struct VersionIndexStore {
    directory: PathBuf,
    identity: StoreIdentity,
    next_file_number: AtomicU64,
}

impl VersionIndexStore {
    fn open(directory: &Path, identity: StoreIdentity) -> Result<Self> {
        fs::create_dir_all(directory)?;
        let mut maximum_file_number = 0u64;
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                continue;
            };
            if let Some(number) = parse_node_file_number(file_name)? {
                maximum_file_number = maximum_file_number.max(number);
            }
        }
        let next_file_number = maximum_file_number
            .checked_add(1)
            .ok_or_else(|| Error::invalid_input("version-index file number exhausted"))?
            .max(1);
        Ok(Self {
            directory: directory.to_path_buf(),
            identity,
            next_file_number: AtomicU64::new(next_file_number),
        })
    }

    fn node_path(&self, file_number: u64) -> PathBuf {
        self.directory.join(format!(
            "{NODE_FILE_PREFIX}{file_number:020}{NODE_FILE_SUFFIX}"
        ))
    }

    fn write_node(&self, node: &IndexNode) -> Result<NodeRef> {
        let mut bytes = encode_node(node, self.identity, 1)?;
        loop {
            let file_number = self
                .next_file_number
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                    current.checked_add(1)
                })
                .map_err(|_| Error::invalid_input("version-index file number exhausted"))?;
            if file_number == 0 {
                return Err(Error::invariant("version-index allocated file number zero"));
            }
            bytes[48..56].copy_from_slice(&file_number.to_le_bytes());
            let header_checksum = *blake3::hash(&bytes[..72]).as_bytes();
            bytes[72..104].copy_from_slice(&header_checksum);
            let node_checksum = *blake3::hash(&bytes[..NODE_CHECKSUM_OFFSET]).as_bytes();
            bytes[NODE_CHECKSUM_OFFSET..].copy_from_slice(&node_checksum);
            let path = self.node_path(file_number);
            match OpenOptions::new().write(true).create_new(true).open(path) {
                Ok(mut file) => {
                    file.write_all(&bytes)?;
                    file.flush()?;
                    return Ok(NodeRef {
                        file_number,
                        offset: 0,
                        checksum: node_checksum,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }
    }

    fn read_node(&self, node_reference: NodeRef) -> Result<IndexNode> {
        if node_reference.offset != 0 || node_reference.file_number == 0 {
            return Err(Error::corruption("version-index node reference is invalid"));
        }
        let path = self.node_path(node_reference.file_number);
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(Error::corruption("reachable version-index node is missing"));
            }
            Err(error) => return Err(error.into()),
        };
        if file.metadata()?.len() != VERSION_INDEX_NODE_BYTES as u64 {
            return Err(Error::corruption("version-index node length is invalid"));
        }
        let mut bytes = [0u8; VERSION_INDEX_NODE_BYTES];
        file.read_exact(&mut bytes)?;
        decode_node(&bytes, self.identity, node_reference)
    }

    fn sync_node(&self, file_number: u64) -> Result<()> {
        File::open(self.node_path(file_number))?.sync_data()?;
        Ok(())
    }

    fn sync_directory(&self) -> Result<()> {
        File::open(&self.directory)?.sync_all()?;
        Ok(())
    }
}

pub struct VersionIndex {
    store: Arc<VersionIndexStore>,
    delta_chains: DeltaChainStore,
    resources: BlinkResourceManager,
    tenant_id: u64,
    maximum_cached_nodes: usize,
    cache: Mutex<NodeCache>,
}

impl VersionIndex {
    pub fn open(
        directory: &Path,
        identity: StoreIdentity,
        resources: BlinkResourceManager,
        tenant_id: u64,
        maximum_cached_nodes: usize,
    ) -> Result<Self> {
        if maximum_cached_nodes == 0 {
            return Err(Error::invalid_input(
                "version-index cache must hold at least one node",
            ));
        }
        Ok(Self {
            store: Arc::new(VersionIndexStore::open(directory, identity)?),
            delta_chains: DeltaChainStore::open(directory, identity, resources.clone(), tenant_id)?,
            resources,
            tenant_id,
            maximum_cached_nodes,
            cache: Mutex::new(NodeCache {
                nodes: HashMap::new(),
                lru: VecDeque::new(),
            }),
        })
    }

    pub fn empty_root(&self) -> VersionIndexRoot {
        VersionIndexRoot::empty()
    }

    pub fn store_identity(&self) -> StoreIdentity {
        self.store.identity
    }

    pub fn delta_chains(&self) -> &DeltaChainStore {
        &self.delta_chains
    }

    pub fn root_from_node_reference(&self, node_reference: NodeRef) -> Result<VersionIndexRoot> {
        let cached = self.load_node(node_reference)?;
        if cached.node.level != MAX_RADIX_LEVEL || cached.node.generation == 0 {
            return Err(Error::corruption("version-index root node is invalid"));
        }
        Ok(VersionIndexRoot {
            generation: cached.node.generation,
            node: Some(node_reference),
        })
    }

    pub fn cached_node_count(&self) -> Result<usize> {
        Ok(self
            .cache
            .lock()
            .map_err(|_| Error::invariant("version-index cache lock poisoned"))?
            .nodes
            .len())
    }

    pub fn root_node_is_rebuildable(&self, root: VersionIndexRoot) -> Result<bool> {
        let Some(node_reference) = root.node else {
            if root.generation != 0 {
                return Err(Error::corruption(
                    "empty version-index root has a generation",
                ));
            }
            return Ok(false);
        };
        let cached = self.load_node(node_reference)?;
        if cached.node.level != MAX_RADIX_LEVEL || cached.node.generation > root.generation {
            return Err(Error::corruption("version-index root node is invalid"));
        }
        Ok(cached.node.kind == NodeFileKind::Spill)
    }

    pub fn pin_root(&self, root: VersionIndexRoot) -> Result<VersionIndexRootPin> {
        let reservation = self.reserve_root_pin()?;
        self.pin_root_reserved(root, reservation)
    }

    pub(crate) fn pin_root_reserved(
        &self,
        root: VersionIndexRoot,
        reservation: Reservation,
    ) -> Result<VersionIndexRootPin> {
        match root.node {
            None if root.generation == 0 => {}
            Some(node_reference) => {
                let cached = self.load_node(node_reference)?;
                if cached.node.level != MAX_RADIX_LEVEL || cached.node.generation > root.generation
                {
                    return Err(Error::corruption("version-index root node is invalid"));
                }
            }
            None => {
                return Err(Error::corruption(
                    "empty version-index root has a generation",
                ));
            }
        }
        Ok(VersionIndexRootPin {
            root,
            _reservation: reservation,
        })
    }

    pub(crate) fn reserve_root_pin(&self) -> Result<Reservation> {
        self.resources
            .reserve(
                self.tenant_id,
                ResourceCategory::Metadata,
                ReservationClass::Foreground,
                ROOT_PIN_RESERVATION_BYTES,
            )
            .map_err(map_resource_error)
    }

    pub(crate) fn pin_inserted_root(
        &self,
        root: VersionIndexRoot,
        reservation: Reservation,
    ) -> Result<VersionIndexRootPin> {
        if root.generation == 0
            || root.node.is_none()
            || reservation.category() != ResourceCategory::Metadata
            || reservation.bytes() < ROOT_PIN_RESERVATION_BYTES
        {
            return Err(Error::invariant(
                "pre-reserved version-index root identity is invalid",
            ));
        }
        Ok(VersionIndexRootPin {
            root,
            _reservation: reservation,
        })
    }

    pub fn promote_root_to_durable(&self, root: VersionIndexRoot) -> Result<VersionIndexRoot> {
        if root.generation == 0 || root.node.is_none() {
            return Err(Error::invalid_input(
                "empty version-index root cannot be made durable",
            ));
        }
        let _write_reservation = self
            .resources
            .reserve(
                self.tenant_id,
                ResourceCategory::WritePreparation,
                ReservationClass::Progress,
                u64::from(MAX_RADIX_LEVEL + 1) * NODE_CACHE_RESERVATION_BYTES,
            )
            .map_err(map_resource_error)?;
        let durable_root = self.copy_node_to_durable(
            root.node.unwrap(),
            MAX_RADIX_LEVEL,
            root.generation,
            root.generation,
        )?;
        self.store.sync_directory()?;
        Ok(VersionIndexRoot {
            generation: root.generation,
            node: Some(durable_root),
        })
    }

    pub fn validate_durable_root(&self, root: VersionIndexRoot) -> Result<()> {
        let Some(node_reference) = root.node else {
            return Err(Error::corruption("durable version-index root is empty"));
        };
        if root.generation == 0 {
            return Err(Error::corruption(
                "durable version-index generation is zero",
            ));
        }
        let _walk_reservation = self
            .resources
            .reserve(
                self.tenant_id,
                ResourceCategory::Metadata,
                ReservationClass::Progress,
                NODE_CACHE_RESERVATION_BYTES,
            )
            .map_err(map_resource_error)?;
        self.validate_durable_node(
            node_reference,
            MAX_RADIX_LEVEL,
            root.generation,
            root.generation,
        )
    }

    fn copy_node_to_durable(
        &self,
        node_reference: NodeRef,
        expected_level: u8,
        parent_generation: u64,
        root_generation: u64,
    ) -> Result<NodeRef> {
        let cached = self.load_node(node_reference)?;
        if cached.node.level != expected_level
            || cached.node.generation > parent_generation
            || cached.node.generation > root_generation
        {
            return Err(Error::corruption(
                "version-index durable copy found invalid ancestry",
            ));
        }
        let mut node = cached.node.clone();
        drop(cached);
        match &mut node.content {
            NodeContent::Internal(children) if expected_level > 0 => {
                for child in children.iter_mut().flatten() {
                    *child = self.copy_node_to_durable(
                        *child,
                        expected_level - 1,
                        node.generation,
                        root_generation,
                    )?;
                }
            }
            NodeContent::Leaf(descriptors) if expected_level == 0 => {
                for (slot, descriptor) in descriptors.iter().enumerate() {
                    if let Some(descriptor) = descriptor
                        && page_nibble(descriptor.page_id, 0) != slot
                    {
                        return Err(Error::corruption(
                            "version-index durable copy found a mismatched leaf key",
                        ));
                    }
                }
            }
            _ => {
                return Err(Error::corruption(
                    "version-index durable copy found a wrong node kind",
                ));
            }
        }
        node.kind = NodeFileKind::Durable;
        let durable_reference = self.store.write_node(&node)?;
        self.store.sync_node(durable_reference.file_number)?;
        Ok(durable_reference)
    }

    fn validate_durable_node(
        &self,
        node_reference: NodeRef,
        expected_level: u8,
        parent_generation: u64,
        root_generation: u64,
    ) -> Result<()> {
        let cached = self.load_node(node_reference)?;
        if cached.node.level != expected_level
            || cached.node.generation > parent_generation
            || cached.node.generation > root_generation
            || cached.node.kind != NodeFileKind::Durable
        {
            return Err(Error::corruption(
                "durable version-index node or ancestry is invalid",
            ));
        }
        match &cached.node.content {
            NodeContent::Internal(children) if expected_level > 0 => {
                let generation = cached.node.generation;
                let child_references = children.iter().flatten().copied().collect::<Vec<_>>();
                drop(cached);
                for child_reference in child_references {
                    self.validate_durable_node(
                        child_reference,
                        expected_level - 1,
                        generation,
                        root_generation,
                    )?;
                }
                Ok(())
            }
            NodeContent::Leaf(_) if expected_level == 0 => Ok(()),
            _ => Err(Error::corruption(
                "durable version-index node has a wrong level",
            )),
        }
    }

    pub fn lookup(
        &self,
        root: VersionIndexRoot,
        page_id: PageId,
    ) -> Result<Option<VersionDescriptor>> {
        let Some(mut node_reference) = root.node else {
            if root.generation != 0 {
                return Err(Error::corruption(
                    "empty version-index root has a generation",
                ));
            }
            return Ok(None);
        };
        let mut expected_level = MAX_RADIX_LEVEL;
        let mut parent_generation = root.generation;
        loop {
            let cached = self.load_node(node_reference)?;
            if cached.node.level != expected_level
                || cached.node.generation > parent_generation
                || cached.node.generation > root.generation
            {
                return Err(Error::corruption(
                    "version-index node level or generation ancestry is invalid",
                ));
            }
            let slot = page_nibble(page_id, expected_level);
            if cached.node.bitmap & (1 << slot) == 0 {
                return Ok(None);
            }
            match &cached.node.content {
                NodeContent::Leaf(descriptors) => {
                    if expected_level != 0 {
                        return Err(Error::corruption("version-index leaf is at a wrong level"));
                    }
                    let descriptor = descriptors[slot]
                        .ok_or_else(|| Error::corruption("version-index leaf slot is empty"))?;
                    if descriptor.page_id != page_id {
                        return Err(Error::corruption(
                            "version-index leaf descriptor does not match its key",
                        ));
                    }
                    return Ok(Some(descriptor));
                }
                NodeContent::Internal(children) => {
                    if expected_level == 0 {
                        return Err(Error::corruption(
                            "version-index internal node is at leaf level",
                        ));
                    }
                    node_reference = children[slot]
                        .ok_or_else(|| Error::corruption("version-index child is missing"))?;
                    parent_generation = cached.node.generation;
                    expected_level -= 1;
                }
            }
        }
    }

    pub fn visit_sources<F>(&self, root: VersionIndexRoot, visitor: &mut F) -> Result<()>
    where
        F: FnMut(SourceReachability) -> Result<()>,
    {
        fn visit_node<F>(
            index: &VersionIndex,
            node_reference: NodeRef,
            expected_level: u8,
            parent_generation: u64,
            root_generation: u64,
            visitor: &mut F,
        ) -> Result<()>
        where
            F: FnMut(SourceReachability) -> Result<()>,
        {
            let cached = index.load_node(node_reference)?;
            if cached.node.level != expected_level
                || cached.node.generation > parent_generation
                || cached.node.generation > root_generation
            {
                return Err(Error::corruption(
                    "version-index traversal level or generation is invalid",
                ));
            }
            match &cached.node.content {
                NodeContent::Leaf(descriptors) if expected_level == 0 => {
                    let descriptors = **descriptors;
                    drop(cached);
                    for descriptor in descriptors.into_iter().flatten() {
                        if descriptor.page_id.get() < 2 {
                            return Err(Error::corruption(
                                "version-index traversal found an invalid page id",
                            ));
                        }
                        match descriptor.locator {
                            VersionLocator::CheckpointImage { source, .. } => {
                                visitor(SourceReachability::Exact(source))?
                            }
                            VersionLocator::WalImage { frame } => {
                                visitor(SourceReachability::Exact(frame.source))?
                            }
                            VersionLocator::WalDelta { frame, chain, .. } => {
                                visitor(SourceReachability::Exact(frame.source))?;
                                let ancestry =
                                    index.delta_chains.read(chain, descriptor.page_id)?;
                                for entry in ancestry.entries {
                                    visitor(SourceReachability::Exact(entry.source_identity()))?;
                                }
                            }
                        }
                    }
                    Ok(())
                }
                NodeContent::Internal(children) if expected_level > 0 => {
                    let generation = cached.node.generation;
                    let child_references = **children;
                    drop(cached);
                    for child_reference in child_references.into_iter().flatten() {
                        visit_node(
                            index,
                            child_reference,
                            expected_level - 1,
                            generation,
                            root_generation,
                            visitor,
                        )?;
                    }
                    Ok(())
                }
                _ => Err(Error::corruption(
                    "version-index traversal found a node at a wrong level",
                )),
            }
        }

        let Some(root_reference) = root.node else {
            if root.generation != 0 {
                return Err(Error::corruption(
                    "empty version-index traversal root has a generation",
                ));
            }
            return Ok(());
        };
        if root.generation == 0 {
            return Err(Error::corruption(
                "version-index traversal root generation is zero",
            ));
        }
        visit_node(
            self,
            root_reference,
            MAX_RADIX_LEVEL,
            root.generation,
            root.generation,
            visitor,
        )
    }

    pub fn visit_descriptors<F>(&self, root: VersionIndexRoot, visitor: &mut F) -> Result<()>
    where
        F: FnMut(VersionDescriptor) -> Result<()>,
    {
        fn visit_node<F>(
            index: &VersionIndex,
            node_reference: NodeRef,
            expected_level: u8,
            parent_generation: u64,
            root_generation: u64,
            key_prefix: u64,
            visitor: &mut F,
        ) -> Result<()>
        where
            F: FnMut(VersionDescriptor) -> Result<()>,
        {
            let cached = index.load_node(node_reference)?;
            if cached.node.level != expected_level
                || cached.node.generation > parent_generation
                || cached.node.generation > root_generation
            {
                return Err(Error::corruption(
                    "version-index descriptor traversal ancestry is invalid",
                ));
            }
            match &cached.node.content {
                NodeContent::Leaf(descriptors) if expected_level == 0 => {
                    let descriptors = **descriptors;
                    drop(cached);
                    for (slot, descriptor) in descriptors.into_iter().enumerate() {
                        if let Some(descriptor) = descriptor {
                            if descriptor.page_id.get() < 2
                                || descriptor.page_id.get() != (key_prefix | slot as u64)
                            {
                                return Err(Error::corruption(
                                    "version-index descriptor traversal found a mismatched key",
                                ));
                            }
                            visitor(descriptor)?;
                        }
                    }
                    Ok(())
                }
                NodeContent::Internal(children) if expected_level > 0 => {
                    let generation = cached.node.generation;
                    let child_references = **children;
                    drop(cached);
                    for (slot, child_reference) in child_references.into_iter().enumerate() {
                        if let Some(child_reference) = child_reference {
                            visit_node(
                                index,
                                child_reference,
                                expected_level - 1,
                                generation,
                                root_generation,
                                key_prefix | ((slot as u64) << (u32::from(expected_level) * 4)),
                                visitor,
                            )?;
                        }
                    }
                    Ok(())
                }
                _ => Err(Error::corruption(
                    "version-index descriptor traversal found a node at a wrong level",
                )),
            }
        }

        let Some(root_reference) = root.node else {
            if root.generation != 0 {
                return Err(Error::corruption(
                    "empty version-index traversal root has a generation",
                ));
            }
            return Ok(());
        };
        if root.generation == 0 {
            return Err(Error::corruption(
                "version-index traversal root generation is zero",
            ));
        }
        visit_node(
            self,
            root_reference,
            MAX_RADIX_LEVEL,
            root.generation,
            root.generation,
            0,
            visitor,
        )
    }

    pub fn insert(
        &self,
        root: VersionIndexRoot,
        page_id: PageId,
        descriptor: VersionDescriptor,
    ) -> Result<VersionIndexRoot> {
        let reservation = self.reserve_insert_work(1)?;
        self.insert_reserved(root, page_id, descriptor, &reservation)
    }

    pub(crate) fn reserve_insert_work(&self, insert_count: usize) -> Result<Reservation> {
        let byte_count = u64::try_from(insert_count)
            .ok()
            .and_then(|count| count.checked_mul(INSERT_RESERVATION_BYTES))
            .ok_or_else(|| Error::overloaded("version-index publication reservation overflows"))?;
        self.resources
            .reserve(
                self.tenant_id,
                ResourceCategory::WritePreparation,
                ReservationClass::Foreground,
                byte_count,
            )
            .map_err(map_resource_error)
    }

    pub(crate) fn insert_reserved(
        &self,
        root: VersionIndexRoot,
        page_id: PageId,
        descriptor: VersionDescriptor,
        reservation: &Reservation,
    ) -> Result<VersionIndexRoot> {
        if reservation.category() != ResourceCategory::WritePreparation
            || reservation.bytes() < INSERT_RESERVATION_BYTES
        {
            return Err(Error::invariant(
                "version-index publication reservation is insufficient",
            ));
        }
        if descriptor.page_id != page_id {
            return Err(Error::invalid_input(
                "version-index descriptor key differs from insert key",
            ));
        }
        validate_descriptor(descriptor)?;
        if let VersionLocator::WalDelta {
            frame,
            chain,
            depth,
        } = descriptor.locator
        {
            let ancestry = self.delta_chains.read(chain, page_id)?;
            let latest_entry = ancestry.entries.last().copied();
            let latest_token = descriptor.token;
            let latest_matches = match (latest_entry, latest_token) {
                (
                    Some(super::version_chain::DeltaChainEntry::WalDelta {
                        frame: latest_frame,
                        record_lsn,
                        image_lsn,
                        image_crc32c,
                        ..
                    }),
                    VersionToken::Wal {
                        source_file_number,
                        record_lsn: token_lsn,
                    },
                ) => {
                    latest_frame == frame
                        && record_lsn == token_lsn
                        && source_file_number == frame.source.file_number
                        && image_lsn == descriptor.image_lsn
                        && image_crc32c == descriptor.image_crc32c
                }
                _ => false,
            };
            if !latest_matches || ancestry.entries.len().saturating_sub(1) != usize::from(depth) {
                return Err(Error::corruption(
                    "delta-chain sidecar does not match its index descriptor",
                ));
            }
        }
        let generation = root
            .generation
            .checked_add(1)
            .ok_or_else(|| Error::invalid_input("version-index generation exhausted"))?;
        let mut path_nodes: [Option<IndexNode>; 16] = std::array::from_fn(|_| None);
        let mut current_reference = root.node;
        let mut parent_generation = root.generation;
        for level in (0..=MAX_RADIX_LEVEL).rev() {
            let Some(node_reference) = current_reference else {
                break;
            };
            let cached = self.load_node(node_reference)?;
            if cached.node.level != level || cached.node.generation > parent_generation {
                return Err(Error::corruption(
                    "version-index path has invalid level or generation ancestry",
                ));
            }
            let slot = page_nibble(page_id, level);
            current_reference = match &cached.node.content {
                NodeContent::Internal(children) if level > 0 => children[slot],
                NodeContent::Leaf(_) if level == 0 => None,
                _ => {
                    return Err(Error::corruption(
                        "version-index node kind does not match its level",
                    ));
                }
            };
            parent_generation = cached.node.generation;
            path_nodes[level as usize] = Some(cached.node.clone());
            if level == 0 {
                break;
            }
        }

        let mut leaf = path_nodes[0]
            .take()
            .unwrap_or_else(|| IndexNode::empty(0, generation, NodeFileKind::Spill));
        let NodeContent::Leaf(descriptors) = &mut leaf.content else {
            return Err(Error::corruption("version-index leaf content is invalid"));
        };
        let leaf_slot = page_nibble(page_id, 0);
        descriptors[leaf_slot] = Some(descriptor);
        leaf.bitmap |= 1 << leaf_slot;
        leaf.generation = generation;
        leaf.kind = NodeFileKind::Spill;
        let mut child_reference = self.store.write_node(&leaf)?;

        for level in 1..=MAX_RADIX_LEVEL {
            let mut parent = path_nodes[level as usize]
                .take()
                .unwrap_or_else(|| IndexNode::empty(level, generation, NodeFileKind::Spill));
            let NodeContent::Internal(children) = &mut parent.content else {
                return Err(Error::corruption(
                    "version-index internal content is invalid",
                ));
            };
            let slot = page_nibble(page_id, level);
            children[slot] = Some(child_reference);
            parent.bitmap |= 1 << slot;
            parent.generation = generation;
            parent.kind = NodeFileKind::Spill;
            child_reference = self.store.write_node(&parent)?;
        }
        Ok(VersionIndexRoot {
            generation,
            node: Some(child_reference),
        })
    }

    pub fn sync_spill_node(&self, node_reference: NodeRef) -> Result<()> {
        let cached = self.load_node(node_reference)?;
        if cached.node.kind != NodeFileKind::Spill {
            return Ok(());
        }
        self.store.sync_node(node_reference.file_number)
    }

    fn load_node(&self, node_reference: NodeRef) -> Result<Arc<CachedNode>> {
        {
            let mut cache = self
                .cache
                .lock()
                .map_err(|_| Error::invariant("version-index cache lock poisoned"))?;
            if let Some(cached) = cache.nodes.get(&node_reference).cloned() {
                touch_lru(&mut cache.lru, node_reference);
                return Ok(cached);
            }
        }
        let mut evicted_nodes = Vec::new();
        loop {
            let cache_full = self
                .cache
                .lock()
                .map_err(|_| Error::invariant("version-index cache lock poisoned"))?
                .nodes
                .len()
                >= self.maximum_cached_nodes;
            if !cache_full {
                break;
            }
            let evicted = {
                let mut cache = self
                    .cache
                    .lock()
                    .map_err(|_| Error::invariant("version-index cache lock poisoned"))?;
                evict_one(&mut cache)
            };
            match evicted {
                Some(evicted) => evicted_nodes.push(evicted),
                None => break,
            }
        }
        drop(evicted_nodes);
        let reservation = match self.resources.reserve(
            self.tenant_id,
            ResourceCategory::Metadata,
            ReservationClass::Foreground,
            NODE_CACHE_RESERVATION_BYTES,
        ) {
            Ok(reservation) => reservation,
            Err(ResourceError::CapacityExceeded) => {
                self.evict_until_one_available()?;
                self.resources
                    .reserve(
                        self.tenant_id,
                        ResourceCategory::Metadata,
                        ReservationClass::Foreground,
                        NODE_CACHE_RESERVATION_BYTES,
                    )
                    .map_err(map_resource_error)?
            }
            Err(error) => return Err(map_resource_error(error)),
        };
        let node = self.store.read_node(node_reference)?;
        let candidate = Arc::new(CachedNode {
            node,
            _reservation: reservation,
        });
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| Error::invariant("version-index cache lock poisoned"))?;
        if let Some(cached) = cache.nodes.get(&node_reference).cloned() {
            touch_lru(&mut cache.lru, node_reference);
            return Ok(cached);
        }
        cache.nodes.insert(node_reference, Arc::clone(&candidate));
        touch_lru(&mut cache.lru, node_reference);
        if cache.nodes.len() > self.maximum_cached_nodes
            && let Some(evicted) = evict_one(&mut cache)
        {
            drop(cache);
            drop(evicted);
            return Ok(candidate);
        }
        Ok(candidate)
    }

    fn evict_until_one_available(&self) -> Result<()> {
        loop {
            let evicted = {
                let mut cache = self
                    .cache
                    .lock()
                    .map_err(|_| Error::invariant("version-index cache lock poisoned"))?;
                evict_one(&mut cache)
            };
            match evicted {
                Some(evicted) => drop(evicted),
                None => {
                    return Err(Error::overloaded(
                        "version-index metadata capacity is held by active node owners",
                    ));
                }
            }
            match self.resources.reserve(
                self.tenant_id,
                ResourceCategory::Metadata,
                ReservationClass::Foreground,
                NODE_CACHE_RESERVATION_BYTES,
            ) {
                Ok(reservation) => {
                    drop(reservation);
                    return Ok(());
                }
                Err(ResourceError::CapacityExceeded) => {}
                Err(error) => return Err(map_resource_error(error)),
            }
        }
    }
}

fn touch_lru(lru: &mut VecDeque<NodeRef>, node_reference: NodeRef) {
    lru.retain(|candidate| *candidate != node_reference);
    lru.push_back(node_reference);
}

fn evict_one(cache: &mut NodeCache) -> Option<Arc<CachedNode>> {
    while let Some(node_reference) = cache.lru.pop_front() {
        if let Some(cached) = cache.nodes.remove(&node_reference) {
            return Some(cached);
        }
    }
    None
}

fn map_resource_error(error: ResourceError) -> Error {
    match error {
        ResourceError::CapacityExceeded | ResourceError::TooManyTenants => {
            Error::overloaded("version-index resource capacity exceeded")
        }
        ResourceError::InvalidLimits => {
            Error::invalid_input("version-index resource limits invalid")
        }
        ResourceError::ArithmeticOverflow => {
            Error::invariant("version-index resource arithmetic overflow")
        }
        ResourceError::TooManyWaiters => Error::overloaded("version-index waiter limit exceeded"),
        ResourceError::SharedOwnership => {
            Error::invariant("version-index reservation ownership invalid")
        }
        ResourceError::WaitTimedOut => Error::overloaded("version-index resource wait timed out"),
        ResourceError::Poisoned => Error::invariant("version-index resource manager lock poisoned"),
    }
}

fn parse_node_file_number(file_name: &str) -> Result<Option<u64>> {
    let Some(number) = file_name
        .strip_prefix(NODE_FILE_PREFIX)
        .and_then(|remaining| remaining.strip_suffix(NODE_FILE_SUFFIX))
    else {
        return Ok(None);
    };
    if number.len() != 20 || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error::corruption(
            "version-index node filename is malformed",
        ));
    }
    let file_number = number
        .parse::<u64>()
        .map_err(|_| Error::corruption("version-index node filename number is invalid"))?;
    Ok(Some(file_number))
}

fn page_nibble(page_id: PageId, level: u8) -> usize {
    let nibble_number = MAX_RADIX_LEVEL - level;
    let shift = (MAX_RADIX_LEVEL - nibble_number) * 4;
    ((page_id.get() >> shift) & 0x0f) as usize
}

fn validate_descriptor(descriptor: VersionDescriptor) -> Result<()> {
    match descriptor.token {
        VersionToken::Bootstrap { page_id } if page_id != descriptor.page_id => {
            return Err(Error::invalid_input(
                "bootstrap version token does not match its page ID",
            ));
        }
        VersionToken::Wal {
            source_file_number,
            record_lsn,
        } if source_file_number == 0 || record_lsn == Lsn::ZERO => {
            return Err(Error::invalid_input("WAL version token is incomplete"));
        }
        _ => {}
    }
    match descriptor.locator {
        VersionLocator::CheckpointImage {
            source,
            offset,
            source_length,
            image_lsn,
            image_crc32c,
        } => {
            validate_source(source)?;
            let image_end = offset
                .checked_add(VERSION_INDEX_NODE_BYTES as u64)
                .ok_or_else(|| Error::invalid_input("checkpoint image range overflows"))?;
            if source.kind != SourceFileKind::CheckpointPack
                || image_lsn != descriptor.image_lsn
                || image_crc32c != descriptor.image_crc32c
                || image_end > source_length
            {
                return Err(Error::invalid_input(
                    "checkpoint locator version metadata differs",
                ));
            }
        }
        VersionLocator::WalImage { frame } => {
            validate_frame(frame)?;
            validate_wal_version_source(descriptor.token, frame.source)?;
        }
        VersionLocator::WalDelta {
            frame,
            chain,
            depth,
        } => {
            validate_frame(frame)?;
            validate_wal_version_source(descriptor.token, frame.source)?;
            if chain.file_number == 0
                || chain.offset != 0
                || chain.checksum == [0; 32]
                || depth == 0
                || depth > 8
            {
                return Err(Error::invalid_input(
                    "WAL delta locator metadata is inconsistent",
                ));
            }
        }
    }
    Ok(())
}

fn validate_wal_version_source(token: VersionToken, source: SourceIdentity) -> Result<()> {
    match token {
        VersionToken::Wal {
            source_file_number, ..
        } if source.kind == SourceFileKind::WalSegment
            && source_file_number == source.file_number =>
        {
            Ok(())
        }
        _ => Err(Error::invalid_input(
            "WAL version token does not match its source file",
        )),
    }
}

fn validate_source(source: SourceIdentity) -> Result<()> {
    if source.file_number == 0 {
        return Err(Error::invalid_input("page source file number is zero"));
    }
    Ok(())
}

fn validate_frame(frame: WalFrameLocator) -> Result<()> {
    validate_source(frame.source)?;
    let frame_end = frame
        .frame_offset
        .checked_add(u64::from(frame.frame_length))
        .ok_or_else(|| Error::invalid_input("WAL frame locator range overflows"))?;
    let payload_end = frame
        .payload_offset
        .checked_add(u64::from(frame.payload_length))
        .ok_or_else(|| Error::invalid_input("WAL payload locator range overflows"))?;
    if frame.frame_length == 0
        || frame.payload_length == 0
        || frame_end > frame.committed_prefix_length
        || payload_end > frame.committed_prefix_length
        || frame.payload_offset < frame.frame_offset
        || payload_end > frame_end
        || frame.commit_frame_offset < frame_end
        || frame.commit_frame_offset >= frame.committed_prefix_length
        || frame.commit_lsn == Lsn::ZERO
    {
        return Err(Error::invalid_input("WAL frame locator bounds are invalid"));
    }
    Ok(())
}

fn encode_node(
    node: &IndexNode,
    identity: StoreIdentity,
    file_number: u64,
) -> Result<[u8; VERSION_INDEX_NODE_BYTES]> {
    if node.level > MAX_RADIX_LEVEL || file_number == 0 {
        return Err(Error::invalid_input("version-index node header is invalid"));
    }
    let mut bytes = [0u8; VERSION_INDEX_NODE_BYTES];
    bytes[0..8].copy_from_slice(&NODE_MAGIC);
    bytes[8..10].copy_from_slice(&NODE_FORMAT_VERSION.to_le_bytes());
    bytes[10..12].copy_from_slice(&node.kind.encode().to_le_bytes());
    bytes[12] = node.level;
    bytes[16..24].copy_from_slice(&node.generation.to_le_bytes());
    bytes[24..40].copy_from_slice(&identity.database_uuid);
    bytes[40..48].copy_from_slice(&identity.wal_epoch.to_le_bytes());
    bytes[48..56].copy_from_slice(&file_number.to_le_bytes());
    bytes[56..58].copy_from_slice(&node.bitmap.to_le_bytes());
    bytes[64..68].copy_from_slice(&(NODE_PAYLOAD_BYTES as u32).to_le_bytes());
    let header_checksum = *blake3::hash(&bytes[..72]).as_bytes();
    bytes[72..104].copy_from_slice(&header_checksum);
    match &node.content {
        NodeContent::Internal(children) if node.level > 0 => {
            let mut observed_bitmap = 0u16;
            let mut child_references = HashSet::new();
            for (slot, child) in children.iter().enumerate() {
                let start = NODE_HEADER_BYTES + slot * CHILD_REFERENCE_BYTES;
                if let Some(child) = child {
                    observed_bitmap |= 1 << slot;
                    if child.file_number == 0
                        || child.offset != 0
                        || !child_references.insert(*child)
                    {
                        return Err(Error::invalid_input(
                            "version-index child reference is invalid",
                        ));
                    }
                    bytes[start..start + 8].copy_from_slice(&child.file_number.to_le_bytes());
                    bytes[start + 8..start + 16].copy_from_slice(&child.offset.to_le_bytes());
                    bytes[start + 16..start + 48].copy_from_slice(&child.checksum);
                }
            }
            if observed_bitmap != node.bitmap {
                return Err(Error::invalid_input(
                    "version-index child bitmap is inconsistent",
                ));
            }
        }
        NodeContent::Leaf(descriptors) if node.level == 0 => {
            let mut observed_bitmap = 0u16;
            for (slot, descriptor) in descriptors.iter().enumerate() {
                let start = NODE_HEADER_BYTES + slot * LEAF_DESCRIPTOR_BYTES;
                if let Some(descriptor) = descriptor {
                    validate_descriptor(*descriptor)?;
                    if page_nibble(descriptor.page_id, 0) != slot {
                        return Err(Error::invalid_input(
                            "version-index descriptor is in the wrong leaf slot",
                        ));
                    }
                    observed_bitmap |= 1 << slot;
                    encode_descriptor(
                        *descriptor,
                        &mut bytes[start..start + LEAF_DESCRIPTOR_BYTES],
                    )?;
                }
            }
            if observed_bitmap != node.bitmap {
                return Err(Error::invalid_input(
                    "version-index leaf bitmap is inconsistent",
                ));
            }
        }
        _ => {
            return Err(Error::invalid_input(
                "version-index node content mismatches its level",
            ));
        }
    }
    let header_checksum = *blake3::hash(&bytes[..72]).as_bytes();
    bytes[72..104].copy_from_slice(&header_checksum);
    let node_checksum = *blake3::hash(&bytes[..NODE_CHECKSUM_OFFSET]).as_bytes();
    bytes[NODE_CHECKSUM_OFFSET..].copy_from_slice(&node_checksum);
    Ok(bytes)
}

fn decode_node(
    bytes: &[u8; VERSION_INDEX_NODE_BYTES],
    identity: StoreIdentity,
    node_reference: NodeRef,
) -> Result<IndexNode> {
    if bytes[0..8] != NODE_MAGIC
        || u16::from_le_bytes(bytes[8..10].try_into().unwrap()) != NODE_FORMAT_VERSION
    {
        return Err(Error::unsupported_format(
            "version-index node magic or version is unsupported",
        ));
    }
    let kind = NodeFileKind::decode(u16::from_le_bytes(bytes[10..12].try_into().unwrap()))?;
    let level = bytes[12];
    if level > MAX_RADIX_LEVEL
        || bytes[13..16].iter().any(|byte| *byte != 0)
        || u64::from_le_bytes(bytes[16..24].try_into().unwrap()) == 0
        || bytes[24..40] != identity.database_uuid
        || u64::from_le_bytes(bytes[40..48].try_into().unwrap()) != identity.wal_epoch
        || u64::from_le_bytes(bytes[48..56].try_into().unwrap()) != node_reference.file_number
        || bytes[58..64].iter().any(|byte| *byte != 0)
        || u32::from_le_bytes(bytes[64..68].try_into().unwrap()) as usize != NODE_PAYLOAD_BYTES
        || bytes[68..72].iter().any(|byte| *byte != 0)
        || bytes[104..NODE_HEADER_BYTES].iter().any(|byte| *byte != 0)
    {
        return Err(Error::corruption(
            "version-index node header identity or reserved fields are invalid",
        ));
    }
    if blake3::hash(&bytes[..72]).as_bytes() != &bytes[72..104]
        || blake3::hash(&bytes[..NODE_CHECKSUM_OFFSET]).as_bytes() != &bytes[NODE_CHECKSUM_OFFSET..]
        || bytes[NODE_CHECKSUM_OFFSET..] != node_reference.checksum
    {
        return Err(Error::corruption("version-index node checksum mismatch"));
    }
    let bitmap = u16::from_le_bytes(bytes[56..58].try_into().unwrap());
    let generation = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
    let payload = &bytes[NODE_HEADER_BYTES..NODE_CHECKSUM_OFFSET];
    let content = if level == 0 {
        let mut descriptors = Box::new(std::array::from_fn(|_| None));
        let mut observed_bitmap = 0u16;
        for slot in 0..16 {
            let start = slot * LEAF_DESCRIPTOR_BYTES;
            let descriptor_bytes = &payload[start..start + LEAF_DESCRIPTOR_BYTES];
            if bitmap & (1 << slot) != 0 {
                let descriptor = decode_descriptor(descriptor_bytes)?;
                if page_nibble(descriptor.page_id, 0) != slot {
                    return Err(Error::corruption(
                        "version-index descriptor is in the wrong leaf slot",
                    ));
                }
                observed_bitmap |= 1 << slot;
                descriptors[slot] = Some(descriptor);
            } else if descriptor_bytes.iter().any(|byte| *byte != 0) {
                return Err(Error::corruption(
                    "empty version-index leaf slot is not zero",
                ));
            }
        }
        if payload[16 * LEAF_DESCRIPTOR_BYTES..]
            .iter()
            .any(|byte| *byte != 0)
            || observed_bitmap != bitmap
        {
            return Err(Error::corruption(
                "version-index leaf payload or bitmap is invalid",
            ));
        }
        NodeContent::Leaf(descriptors)
    } else {
        let mut children = Box::new(std::array::from_fn(|_| None));
        let mut observed_bitmap = 0u16;
        let mut unique_references = HashSet::new();
        for slot in 0..16 {
            let start = slot * CHILD_REFERENCE_BYTES;
            let child_bytes = &payload[start..start + CHILD_REFERENCE_BYTES];
            if bitmap & (1 << slot) != 0 {
                let child = NodeRef {
                    file_number: u64::from_le_bytes(child_bytes[0..8].try_into().unwrap()),
                    offset: u64::from_le_bytes(child_bytes[8..16].try_into().unwrap()),
                    checksum: child_bytes[16..48].try_into().unwrap(),
                };
                if child.file_number == 0 || child.offset != 0 || !unique_references.insert(child) {
                    return Err(Error::corruption(
                        "version-index child reference is invalid",
                    ));
                }
                children[slot] = Some(child);
                observed_bitmap |= 1 << slot;
            } else if child_bytes.iter().any(|byte| *byte != 0) {
                return Err(Error::corruption(
                    "empty version-index child slot is not zero",
                ));
            }
        }
        if payload[16 * CHILD_REFERENCE_BYTES..]
            .iter()
            .any(|byte| *byte != 0)
            || observed_bitmap != bitmap
        {
            return Err(Error::corruption(
                "version-index internal payload or bitmap is invalid",
            ));
        }
        NodeContent::Internal(children)
    };
    Ok(IndexNode {
        level,
        generation,
        bitmap,
        kind,
        content,
    })
}

fn encode_descriptor(descriptor: VersionDescriptor, bytes: &mut [u8]) -> Result<()> {
    if bytes.len() != LEAF_DESCRIPTOR_BYTES {
        return Err(Error::invariant(
            "version-index descriptor buffer has a wrong size",
        ));
    }
    bytes.fill(0);
    bytes[0..8].copy_from_slice(&descriptor.page_id.get().to_le_bytes());
    match descriptor.token {
        VersionToken::Bootstrap { page_id } => {
            bytes[8] = 1;
            bytes[16..24].copy_from_slice(&page_id.get().to_le_bytes());
        }
        VersionToken::Wal {
            source_file_number,
            record_lsn,
        } => {
            bytes[8] = 2;
            bytes[16..24].copy_from_slice(&source_file_number.to_le_bytes());
            bytes[24..32].copy_from_slice(&record_lsn.get().to_le_bytes());
        }
    }
    bytes[32..40].copy_from_slice(&descriptor.image_lsn.get().to_le_bytes());
    bytes[40..44].copy_from_slice(&descriptor.image_crc32c.to_le_bytes());
    match descriptor.locator {
        VersionLocator::CheckpointImage {
            source,
            offset,
            source_length,
            image_lsn,
            image_crc32c,
        } => {
            bytes[9] = 1;
            encode_source(source, bytes)?;
            bytes[88..96].copy_from_slice(&offset.to_le_bytes());
            bytes[104..112].copy_from_slice(&offset.to_le_bytes());
            bytes[112..116].copy_from_slice(&(VERSION_INDEX_NODE_BYTES as u32).to_le_bytes());
            bytes[116..120].copy_from_slice(&image_crc32c.to_le_bytes());
            bytes[120..128].copy_from_slice(&source_length.to_le_bytes());
            if image_lsn != descriptor.image_lsn || image_crc32c != descriptor.image_crc32c {
                return Err(Error::invalid_input(
                    "checkpoint locator version differs from descriptor",
                ));
            }
        }
        VersionLocator::WalImage { frame } => {
            bytes[9] = 2;
            encode_frame_locator(frame, bytes)?;
        }
        VersionLocator::WalDelta {
            frame,
            chain,
            depth,
        } => {
            bytes[9] = 3;
            bytes[10] = depth;
            encode_frame_locator(frame, bytes)?;
            bytes[144..152].copy_from_slice(&chain.file_number.to_le_bytes());
            bytes[152..160].copy_from_slice(&chain.offset.to_le_bytes());
            bytes[160..192].copy_from_slice(&chain.checksum);
        }
    }
    Ok(())
}

fn encode_source(source: SourceIdentity, bytes: &mut [u8]) -> Result<()> {
    validate_source(source)?;
    bytes[11] = source.kind as u8;
    bytes[48..56].copy_from_slice(&source.file_number.to_le_bytes());
    bytes[56..88].copy_from_slice(&source.header_identity);
    Ok(())
}

fn encode_frame_locator(frame: WalFrameLocator, bytes: &mut [u8]) -> Result<()> {
    validate_frame(frame)?;
    encode_source(frame.source, bytes)?;
    bytes[88..96].copy_from_slice(&frame.frame_offset.to_le_bytes());
    bytes[96..100].copy_from_slice(&frame.frame_length.to_le_bytes());
    bytes[100..104].copy_from_slice(&frame.header_crc32c.to_le_bytes());
    bytes[104..112].copy_from_slice(&frame.payload_offset.to_le_bytes());
    bytes[112..116].copy_from_slice(&frame.payload_length.to_le_bytes());
    bytes[116..120].copy_from_slice(&frame.payload_crc32c.to_le_bytes());
    bytes[120..128].copy_from_slice(&frame.committed_prefix_length.to_le_bytes());
    bytes[128..136].copy_from_slice(&frame.commit_frame_offset.to_le_bytes());
    bytes[136..144].copy_from_slice(&frame.commit_lsn.get().to_le_bytes());
    Ok(())
}

fn decode_descriptor(bytes: &[u8]) -> Result<VersionDescriptor> {
    if bytes.len() != LEAF_DESCRIPTOR_BYTES
        || bytes[12..16].iter().any(|byte| *byte != 0)
        || bytes[44..48].iter().any(|byte| *byte != 0)
        || bytes[157..160].iter().any(|byte| *byte != 0)
        || bytes[228..].iter().any(|byte| *byte != 0)
    {
        return Err(Error::corruption(
            "version-index descriptor reserved bytes are invalid",
        ));
    }
    let page_id = PageId::new(u64::from_le_bytes(bytes[0..8].try_into().unwrap()));
    let token = match bytes[8] {
        1 => {
            if bytes[24..32].iter().any(|byte| *byte != 0) {
                return Err(Error::corruption("bootstrap token has a WAL record LSN"));
            }
            VersionToken::Bootstrap {
                page_id: PageId::new(u64::from_le_bytes(bytes[16..24].try_into().unwrap())),
            }
        }
        2 => VersionToken::Wal {
            source_file_number: u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
            record_lsn: Lsn::new(u64::from_le_bytes(bytes[24..32].try_into().unwrap())),
        },
        _ => return Err(Error::corruption("version-index token tag is unknown")),
    };
    let image_lsn = Lsn::new(u64::from_le_bytes(bytes[32..40].try_into().unwrap()));
    let image_crc32c = u32::from_le_bytes(bytes[40..44].try_into().unwrap());
    let locator = match bytes[9] {
        1 => {
            let source = decode_source(bytes)?;
            if bytes[96..104].iter().any(|byte| *byte != 0)
                || bytes[128..144].iter().any(|byte| *byte != 0)
                || bytes[144..].iter().any(|byte| *byte != 0)
                || bytes[10] != 0
            {
                return Err(Error::corruption("checkpoint locator has WAL-only fields"));
            }
            VersionLocator::CheckpointImage {
                source,
                offset: u64::from_le_bytes(bytes[88..96].try_into().unwrap()),
                source_length: u64::from_le_bytes(bytes[120..128].try_into().unwrap()),
                image_lsn: Lsn::new(u64::from_le_bytes(bytes[32..40].try_into().unwrap())),
                image_crc32c: u32::from_le_bytes(bytes[40..44].try_into().unwrap()),
            }
        }
        2 => {
            if bytes[10] != 0 || bytes[144..].iter().any(|byte| *byte != 0) {
                return Err(Error::corruption("WAL image locator has delta-only fields"));
            }
            VersionLocator::WalImage {
                frame: decode_frame_locator(bytes)?,
            }
        }
        3 if bytes[192..].iter().all(|byte| *byte == 0) => VersionLocator::WalDelta {
            frame: decode_frame_locator(bytes)?,
            chain: DeltaChainRef {
                file_number: u64::from_le_bytes(bytes[144..152].try_into().unwrap()),
                offset: u64::from_le_bytes(bytes[152..160].try_into().unwrap()),
                checksum: bytes[160..192].try_into().unwrap(),
            },
            depth: bytes[10],
        },
        _ => return Err(Error::corruption("version-index locator tag is unknown")),
    };
    let descriptor = VersionDescriptor {
        page_id,
        token,
        image_lsn,
        image_crc32c,
        locator,
    };
    validate_descriptor(descriptor)
        .map_err(|_| Error::corruption("version-index descriptor is invalid"))?;
    Ok(descriptor)
}

fn decode_source(bytes: &[u8]) -> Result<SourceIdentity> {
    let source = SourceIdentity {
        kind: decode_source_kind(bytes[11])?,
        file_number: u64::from_le_bytes(bytes[48..56].try_into().unwrap()),
        header_identity: bytes[56..88].try_into().unwrap(),
    };
    validate_source(source)
        .map_err(|_| Error::corruption("version-index source identity is invalid"))?;
    Ok(source)
}

fn decode_source_kind(value: u8) -> Result<SourceFileKind> {
    match value {
        1 => Ok(SourceFileKind::CheckpointPack),
        2 => Ok(SourceFileKind::WalSegment),
        _ => Err(Error::corruption(
            "version-index source file kind is unknown",
        )),
    }
}

fn decode_frame_locator(bytes: &[u8]) -> Result<WalFrameLocator> {
    let frame = WalFrameLocator {
        source: decode_source(bytes)?,
        frame_offset: u64::from_le_bytes(bytes[88..96].try_into().unwrap()),
        frame_length: u32::from_le_bytes(bytes[96..100].try_into().unwrap()),
        header_crc32c: u32::from_le_bytes(bytes[100..104].try_into().unwrap()),
        payload_offset: u64::from_le_bytes(bytes[104..112].try_into().unwrap()),
        payload_length: u32::from_le_bytes(bytes[112..116].try_into().unwrap()),
        payload_crc32c: u32::from_le_bytes(bytes[116..120].try_into().unwrap()),
        committed_prefix_length: u64::from_le_bytes(bytes[120..128].try_into().unwrap()),
        commit_frame_offset: u64::from_le_bytes(bytes[128..136].try_into().unwrap()),
        commit_lsn: Lsn::new(u64::from_le_bytes(bytes[136..144].try_into().unwrap())),
    };
    validate_frame(frame).map_err(|_| Error::corruption("version-index WAL locator is invalid"))?;
    Ok(frame)
}

#[cfg(test)]
mod tests {
    use super::{
        DeltaChainRef, SourceFileKind, SourceIdentity, VersionDescriptor, VersionIndex,
        VersionIndexRoot, VersionLocator, VersionToken, WalFrameLocator, decode_descriptor,
        encode_descriptor,
    };
    use crate::blink::resource::{BlinkResourceManager, ResourceCategory, ResourceLimits};
    use dodb_core::{Lsn, PageId};
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new(label: &str) -> Self {
            let sequence = DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let directory = std::env::temp_dir().join(format!(
                "dodb-version-index-{label}-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&directory).unwrap();
            Self(directory)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn source(kind: SourceFileKind, file_number: u64) -> SourceIdentity {
        SourceIdentity {
            kind,
            file_number,
            header_identity: [file_number as u8; 32],
        }
    }

    fn wal_frame(file_number: u64) -> WalFrameLocator {
        WalFrameLocator {
            source: source(SourceFileKind::WalSegment, file_number),
            frame_offset: 0,
            frame_length: 128,
            header_crc32c: 0x1234_5678,
            payload_offset: 32,
            payload_length: 64,
            payload_crc32c: 0x8765_4321,
            committed_prefix_length: 256,
            commit_frame_offset: 128,
            commit_lsn: Lsn::new(20),
        }
    }

    fn checkpoint_descriptor(page_id: PageId) -> VersionDescriptor {
        VersionDescriptor {
            page_id,
            token: VersionToken::Bootstrap { page_id },
            image_lsn: Lsn::new(10),
            image_crc32c: 0x1020_3040,
            locator: VersionLocator::CheckpointImage {
                source: source(SourceFileKind::CheckpointPack, 3),
                offset: 4096,
                source_length: 16384,
                image_lsn: Lsn::new(10),
                image_crc32c: 0x1020_3040,
            },
        }
    }

    fn wal_image_descriptor(page_id: PageId) -> VersionDescriptor {
        VersionDescriptor {
            page_id,
            token: VersionToken::Wal {
                source_file_number: 7,
                record_lsn: Lsn::new(18),
            },
            image_lsn: Lsn::new(19),
            image_crc32c: 0x1122_3344,
            locator: VersionLocator::WalImage {
                frame: wal_frame(7),
            },
        }
    }

    fn wal_delta_descriptor(page_id: PageId) -> VersionDescriptor {
        VersionDescriptor {
            page_id,
            token: VersionToken::Wal {
                source_file_number: 8,
                record_lsn: Lsn::new(30),
            },
            image_lsn: Lsn::new(31),
            image_crc32c: 0x5566_7788,
            locator: VersionLocator::WalDelta {
                frame: wal_frame(8),
                chain: DeltaChainRef {
                    file_number: 9,
                    offset: 0,
                    checksum: [0x88; 32],
                },
                depth: 3,
            },
        }
    }

    fn indexed_wal_delta_descriptor(index: &VersionIndex, page_id: PageId) -> VersionDescriptor {
        let mut descriptor = wal_delta_descriptor(page_id);
        let VersionLocator::WalDelta { frame, .. } = descriptor.locator else {
            unreachable!();
        };
        let chain = super::super::version_chain::DeltaChain::new(
            page_id,
            vec![
                super::super::version_chain::DeltaChainEntry::CheckpointImage {
                    source: source(SourceFileKind::CheckpointPack, 4),
                    offset: 8192,
                    source_length: 16384,
                    image_lsn: Lsn::new(25),
                    image_crc32c: 0x3344_5566,
                },
                super::super::version_chain::DeltaChainEntry::WalDelta {
                    frame,
                    record_lsn: Lsn::new(30),
                    base_lsn: Lsn::new(25),
                    base_crc32c: 0x3344_5566,
                    image_lsn: Lsn::new(31),
                    image_crc32c: 0x5566_7788,
                },
            ],
        )
        .unwrap();
        let chain = index.delta_chains().write(1, &chain).unwrap();
        descriptor.locator = VersionLocator::WalDelta {
            frame,
            chain,
            depth: 1,
        };
        descriptor
    }

    fn resource_manager() -> BlinkResourceManager {
        BlinkResourceManager::new(ResourceLimits {
            global_bytes: 2 * 1024 * 1024,
            per_tenant_bytes: 2 * 1024 * 1024,
            progress_pool_bytes: 256 * 1024,
            max_tenants: 4,
            max_pending_waiters: 4,
            global_category_bytes: [2 * 1024 * 1024; 7],
            per_tenant_category_bytes: [2 * 1024 * 1024; 7],
        })
        .unwrap()
    }

    fn open_index(
        directory: &Path,
        identity: ([u8; 16], u64),
        maximum_cached_nodes: usize,
    ) -> VersionIndex {
        open_index_with_manager(
            directory,
            identity,
            maximum_cached_nodes,
            resource_manager(),
        )
    }

    fn open_index_with_manager(
        directory: &Path,
        identity: ([u8; 16], u64),
        maximum_cached_nodes: usize,
        resources: BlinkResourceManager,
    ) -> VersionIndex {
        VersionIndex::open(
            directory,
            super::StoreIdentity {
                database_uuid: identity.0,
                wal_epoch: identity.1,
            },
            resources,
            41,
            maximum_cached_nodes,
        )
        .unwrap()
    }

    #[test]
    fn descriptor_encodings_round_trip_all_locator_variants() {
        let descriptors = [
            checkpoint_descriptor(PageId::new(0x0123_4567_89ab_cdef)),
            wal_image_descriptor(PageId::new(0x1020_3040_5060_7080)),
            wal_delta_descriptor(PageId::new(0xfedc_ba98_7654_3210)),
        ];
        for descriptor in descriptors {
            let mut bytes = [0u8; super::LEAF_DESCRIPTOR_BYTES];
            encode_descriptor(descriptor, &mut bytes).unwrap();
            assert_eq!(decode_descriptor(&bytes).unwrap(), descriptor);
        }
    }

    #[test]
    fn sparse_page_ids_survive_reopen_and_old_roots_remain_immutable() {
        let directory = TestDirectory::new("roots");
        let identity = ([9; 16], 12);
        let index = open_index(directory.path(), identity, 3);
        let empty_root = index.empty_root();
        let first_page = PageId::new(0x0000_0000_0000_000f);
        let maximum_page = PageId::new(u64::MAX);
        let first_root = index
            .insert(empty_root, first_page, checkpoint_descriptor(first_page))
            .unwrap();
        let second_root = index
            .insert(first_root, maximum_page, wal_image_descriptor(maximum_page))
            .unwrap();

        assert_eq!(index.lookup(empty_root, first_page).unwrap(), None);
        assert_eq!(
            index.lookup(first_root, first_page).unwrap(),
            Some(checkpoint_descriptor(first_page))
        );
        assert_eq!(index.lookup(first_root, maximum_page).unwrap(), None);
        assert_eq!(
            index.lookup(second_root, first_page).unwrap(),
            Some(checkpoint_descriptor(first_page))
        );
        assert_eq!(
            index.lookup(second_root, maximum_page).unwrap(),
            Some(wal_image_descriptor(maximum_page))
        );

        let durable_reference = second_root.node_reference().unwrap();
        drop(index);
        let reopened = open_index(directory.path(), identity, 2);
        let restored_root =
            VersionIndexRoot::from_node_reference(second_root.generation(), durable_reference)
                .unwrap();
        assert_eq!(
            reopened.lookup(restored_root, first_page).unwrap(),
            Some(checkpoint_descriptor(first_page))
        );
        assert_eq!(
            reopened.lookup(restored_root, maximum_page).unwrap(),
            Some(wal_image_descriptor(maximum_page))
        );
    }

    #[test]
    fn page_reuse_keeps_distinct_tokens_in_pinned_roots() {
        let directory = TestDirectory::new("reuse");
        let index = open_index(directory.path(), ([8; 16], 11), 2);
        let reused_page = PageId::new(0x7654_3210);
        let original_version = checkpoint_descriptor(reused_page);
        let reused_version = wal_image_descriptor(reused_page);
        let original_root = index
            .insert(index.empty_root(), reused_page, original_version)
            .unwrap();
        let reused_root = index
            .insert(original_root, reused_page, reused_version)
            .unwrap();

        assert_ne!(original_version.token, reused_version.token);
        assert_eq!(
            index.lookup(original_root, reused_page).unwrap(),
            Some(original_version)
        );
        assert_eq!(
            index.lookup(reused_root, reused_page).unwrap(),
            Some(reused_version)
        );
    }

    #[test]
    fn many_updates_match_reference_map_with_bounded_node_cache() {
        let directory = TestDirectory::new("cache");
        let resources = resource_manager();
        let index = open_index_with_manager(directory.path(), ([2; 16], 5), 2, resources.clone());
        let mut root = index.empty_root();
        let mut reference = BTreeMap::new();
        let mut retained_root_handles = Vec::new();
        for update_number in 0..28u64 {
            let page_value = update_number.wrapping_mul(0x9e37_79b9_7f4a_7c15);
            let page_id = PageId::new(page_value);
            let descriptor = if page_id.get() >= 2 && update_number % 3 == 0 {
                indexed_wal_delta_descriptor(&index, page_id)
            } else {
                checkpoint_descriptor(page_id)
            };
            root = index.insert(root, page_id, descriptor).unwrap();
            reference.insert(page_id.get(), descriptor);
            retained_root_handles.push(root);
        }
        for page_value in
            (0..28u64).map(|update_number| update_number.wrapping_mul(0x9e37_79b9_7f4a_7c15))
        {
            let page_id = PageId::new(page_value);
            assert_eq!(
                index.lookup(root, page_id).unwrap(),
                reference.get(&page_value).copied()
            );
        }
        assert_eq!(index.cached_node_count().unwrap(), 2);
        assert_eq!(retained_root_handles.len(), 28);
        assert!(fs::read_dir(directory.path()).unwrap().count() >= 28 * 16);
        assert_eq!(
            resources.snapshot().unwrap().current_by_category[ResourceCategory::Metadata as usize],
            2 * super::NODE_CACHE_RESERVATION_BYTES,
        );
    }

    #[test]
    fn retained_root_pins_consume_the_shared_metadata_limit() {
        let directory = TestDirectory::new("root-pins");
        let metadata_limit =
            super::NODE_CACHE_RESERVATION_BYTES + 4 * super::ROOT_PIN_RESERVATION_BYTES;
        let mut global_category_bytes = [2 * 1024 * 1024; 7];
        global_category_bytes[ResourceCategory::Metadata as usize] = metadata_limit;
        let mut per_tenant_category_bytes = [2 * 1024 * 1024; 7];
        per_tenant_category_bytes[ResourceCategory::Metadata as usize] = metadata_limit;
        let resources = BlinkResourceManager::new(ResourceLimits {
            global_bytes: 2 * 1024 * 1024,
            per_tenant_bytes: 2 * 1024 * 1024,
            progress_pool_bytes: 256 * 1024,
            max_tenants: 4,
            max_pending_waiters: 4,
            global_category_bytes,
            per_tenant_category_bytes,
        })
        .unwrap();
        let index = open_index_with_manager(directory.path(), ([6; 16], 22), 1, resources.clone());
        let page_id = PageId::new(0x1020_3040);
        let root = index
            .insert(index.empty_root(), page_id, checkpoint_descriptor(page_id))
            .unwrap();
        let root_pins = (0..4)
            .map(|_| index.pin_root(root).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(root_pins.len(), 4);
        assert_eq!(root_pins[0].root(), root);
        assert!(index.pin_root(root).is_err());
        assert_eq!(
            resources.snapshot().unwrap().current_by_category[ResourceCategory::Metadata as usize],
            metadata_limit,
        );
        drop(root_pins);
        assert_eq!(
            resources.snapshot().unwrap().current_by_category[ResourceCategory::Metadata as usize],
            super::NODE_CACHE_RESERVATION_BYTES,
        );
    }

    #[test]
    fn durable_root_promotion_syncs_the_full_reachable_tree() {
        let directory = TestDirectory::new("durable-root");
        let identity = ([7; 16], 31);
        let index = open_index(directory.path(), identity, 2);
        let mut spill_root = index.empty_root();
        for page_value in [0x0123_4567_89ab_cdef, u64::MAX] {
            let page_id = PageId::new(page_value);
            spill_root = index
                .insert(spill_root, page_id, checkpoint_descriptor(page_id))
                .unwrap();
        }
        assert!(index.validate_durable_root(spill_root).is_err());
        let durable_root = index.promote_root_to_durable(spill_root).unwrap();
        index.validate_durable_root(durable_root).unwrap();
        for page_value in [0x0123_4567_89ab_cdef, u64::MAX] {
            let page_id = PageId::new(page_value);
            assert_eq!(
                index.lookup(durable_root, page_id).unwrap(),
                Some(checkpoint_descriptor(page_id))
            );
            assert_eq!(
                index.lookup(spill_root, page_id).unwrap(),
                Some(checkpoint_descriptor(page_id))
            );
        }
        let durable_reference = durable_root.node_reference().unwrap();
        drop(index);
        let reopened = open_index(directory.path(), identity, 1);
        let reopened_root =
            VersionIndexRoot::from_node_reference(durable_root.generation(), durable_reference)
                .unwrap();
        reopened.validate_durable_root(reopened_root).unwrap();
    }

    #[test]
    fn wrong_store_identity_and_corrupt_node_fail_closed() {
        let directory = TestDirectory::new("corruption");
        let identity = ([3; 16], 6);
        let index = open_index(directory.path(), identity, 2);
        let page_id = PageId::new(0x1234);
        let root = index
            .insert(index.empty_root(), page_id, checkpoint_descriptor(page_id))
            .unwrap();
        let node_reference = root.node_reference().unwrap();
        assert!(index.root_node_is_rebuildable(root).unwrap());
        drop(index);

        let wrong_store = open_index(directory.path(), ([4; 16], 6), 2);
        let wrong_root =
            VersionIndexRoot::from_node_reference(root.generation(), node_reference).unwrap();
        assert!(wrong_store.lookup(wrong_root, page_id).is_err());
        drop(wrong_store);

        let root_file = directory.path().join(format!(
            "version-index-{:020}.node",
            node_reference.file_number
        ));
        let mut bytes = fs::read(&root_file).unwrap();
        bytes[super::NODE_HEADER_BYTES + 1] ^= 0x80;
        fs::write(&root_file, bytes).unwrap();
        let reopened = open_index(directory.path(), identity, 2);
        assert!(reopened.lookup(wrong_root, page_id).is_err());
    }

    #[test]
    fn truncated_node_and_generation_overflow_fail_closed() {
        let directory = TestDirectory::new("truncated");
        let index = open_index(directory.path(), ([5; 16], 17), 2);
        let page_id = PageId::new(0xaabb_ccdd);
        let root = index
            .insert(index.empty_root(), page_id, checkpoint_descriptor(page_id))
            .unwrap();
        let node_reference = root.node_reference().unwrap();
        let overflow_root = VersionIndexRoot {
            generation: u64::MAX,
            node: Some(node_reference),
        };
        assert!(
            index
                .insert(overflow_root, page_id, checkpoint_descriptor(page_id))
                .is_err()
        );
        drop(index);

        let root_file = directory.path().join(format!(
            "version-index-{:020}.node",
            node_reference.file_number
        ));
        fs::write(&root_file, [0u8; 17]).unwrap();
        let reopened = open_index(directory.path(), ([5; 16], 17), 2);
        let restored_root =
            VersionIndexRoot::from_node_reference(root.generation(), node_reference).unwrap();
        assert!(reopened.lookup(restored_root, page_id).is_err());
    }

    #[test]
    fn wrong_level_and_generation_ancestry_fail_closed() {
        let directory = TestDirectory::new("ancestry");
        let index = open_index(directory.path(), ([1; 16], 19), 2);
        let page_id = PageId::new(0x4567_89ab_cdef_0123);
        let wrong_level = index
            .store
            .write_node(&super::IndexNode::empty(0, 1, super::NodeFileKind::Spill))
            .unwrap();
        let wrong_level_root = VersionIndexRoot {
            generation: 1,
            node: Some(wrong_level),
        };
        assert!(index.lookup(wrong_level_root, page_id).is_err());

        let mut descendant = index
            .store
            .write_node(&super::IndexNode::empty(0, 3, super::NodeFileKind::Spill))
            .unwrap();
        for level in 1..=super::MAX_RADIX_LEVEL {
            let mut parent = super::IndexNode::empty(level, 2, super::NodeFileKind::Spill);
            let super::NodeContent::Internal(children) = &mut parent.content else {
                unreachable!();
            };
            let slot = super::page_nibble(page_id, level);
            children[slot] = Some(descendant);
            parent.bitmap |= 1 << slot;
            descendant = index.store.write_node(&parent).unwrap();
        }
        let wrong_ancestry_root = VersionIndexRoot {
            generation: 2,
            node: Some(descendant),
        };
        assert!(index.lookup(wrong_ancestry_root, page_id).is_err());
    }
}
