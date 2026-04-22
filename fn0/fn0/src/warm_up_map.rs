use bytes::{Buf, BufMut, Bytes, BytesMut};
use memberlist::{
    delegate::{EventDelegate, NodeDelegate},
    proto::NodeState,
};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
    sync::Arc,
};
use tokio::sync::RwLock;

type CodeId = u128;
type NodeId = u128;

struct WarmUpMapNode {
    map: RwLock<BTreeMap<CodeId, BTreeSet<NodePresence>>>,
    event_queue: RwLock<BTreeMap<CodeId, BTreeSet<NodePresence>>>,
}

#[derive(Clone, Debug)]
struct NodePresence {
    id: NodeId,
    addr: Ipv4Addr,
    updated_at: u64,
    is_alive: bool,
}
impl PartialEq for NodePresence {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }
}
impl Eq for NodePresence {}
impl PartialOrd for NodePresence {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for NodePresence {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.addr.cmp(&other.addr)
    }
}

impl EventDelegate for WarmUpMapNode {
    type Id = NodeId;
    type Address = Ipv4Addr;

    async fn notify_join(&self, node: Arc<NodeState<Self::Id, Self::Address>>) {
        let mut map = self.map.write().await;
        let id = node.id;
        let addr = node.addr;
        for set in map.values_mut() {
            set.retain(|node| !(node.addr == addr && node.id != id));
        }
    }
    async fn notify_update(&self, _node: Arc<NodeState<Self::Id, Self::Address>>) {}

    async fn notify_leave(&self, node: Arc<NodeState<Self::Id, Self::Address>>) {
        let mut map = self.map.write().await;
        let id = node.id;
        for set in map.values_mut() {
            set.retain(|node| node.id != id);
        }
    }
}

impl NodeDelegate for WarmUpMapNode {
    async fn notify_message(&self, msg: Cow<'_, [u8]>) {
        /*
        message format
        - loop [
            node_id: u128
            ipv4 address: u32
            code id: u128
            updated_at: u64
            is_alive(alive: 1, dead: 0): u8
        ]
        */

        let mut map = self.map.write().await;
        let mut event_map = self.event_queue.write().await;
        let mut reader = msg.as_ref();
        while !reader.is_empty() {
            let id = reader.get_u128();
            let addr = Ipv4Addr::from(reader.get_u32());
            let code_id = reader.get_u128();
            let updated_at = reader.get_u64();
            let is_alive = reader.get_u8() == 1;

            let nodes = map.entry(code_id).or_default();

            let node_from_msg = NodePresence {
                id,
                addr,
                updated_at,
                is_alive,
            };

            let node_in_memory = nodes.get(&node_from_msg);
            let is_updated = match node_in_memory {
                Some(node_in_memory) => {
                    node_in_memory.is_alive != is_alive || node_in_memory.updated_at < updated_at
                }
                None => true,
            };

            if is_updated {
                event_map
                    .entry(code_id)
                    .or_default()
                    .replace(node_from_msg.clone());
                nodes.replace(node_from_msg);
            }
        }
    }

    async fn broadcast_messages<F>(
        &self,
        limit: usize,
        encoded_len: F,
    ) -> impl Iterator<Item = Bytes> + Send
    where
        F: Fn(Bytes) -> (usize, Bytes) + Send + Sync + 'static,
    {
        let mut total_size = 0;
        let mut event_map = self.event_queue.write().await;
        let mut vec = vec![];
        'out: {
            while let Some(mut entry) = event_map.first_entry() {
                let code_id = *entry.key();
                let set = entry.get_mut();
                while let Some(node) = set.pop_first() {
                    let mut bytes = BytesMut::with_capacity(45);
                    bytes.put_u128(node.id);
                    bytes.put_u32(node.addr.to_bits());
                    bytes.put_u128(code_id);
                    bytes.put_u64(node.updated_at);
                    bytes.put_u8(if node.is_alive { 1 } else { 0 });
                    let bytes = bytes.freeze();

                    let (len, bytes) = encoded_len(bytes);

                    if total_size + len > limit {
                        set.insert(node);
                        break 'out;
                    }

                    total_size += len;
                    vec.push(bytes);
                }
                entry.remove();
            }
        }

        vec.into_iter()
    }

    async fn local_state(&self, _join: bool) -> Bytes {
        let map = self.map.read().await;
        let mut bytes =
            BytesMut::with_capacity(map.values().map(|nodes| nodes.len()).sum::<usize>() * 45);
        for (code_id, set) in map.iter() {
            for node in set {
                bytes.put_u128(node.id);
                bytes.put_u32(node.addr.to_bits());
                bytes.put_u128(*code_id);
                bytes.put_u64(node.updated_at);
                bytes.put_u8(if node.is_alive { 1 } else { 0 });
            }
        }

        bytes.freeze()
    }

    async fn merge_remote_state(&self, buf: &[u8], _join: bool) {
        NodeDelegate::notify_message(self, buf.into()).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    mod node_precense_tests {
        use super::*;

        #[test]
        fn test_equality_based_on_addr() {
            let node1 = NodePresence {
                id: 1,
                addr: Ipv4Addr::new(192, 168, 1, 1),
                updated_at: 100,
                is_alive: true,
            };
            let node2 = NodePresence {
                id: 2,
                addr: Ipv4Addr::new(192, 168, 1, 1),
                updated_at: 200,
                is_alive: false,
            };
            let node3 = NodePresence {
                id: 3,
                addr: Ipv4Addr::new(192, 168, 1, 2),
                updated_at: 100,
                is_alive: true,
            };

            assert_eq!(node1, node2);
            assert_ne!(node1, node3);
        }

        #[test]
        fn test_ordering_based_on_addr() {
            let node1 = NodePresence {
                id: 1,
                addr: Ipv4Addr::new(192, 168, 1, 1),
                updated_at: 100,
                is_alive: true,
            };
            let node2 = NodePresence {
                id: 2,
                addr: Ipv4Addr::new(192, 168, 1, 2),
                updated_at: 50,
                is_alive: false,
            };

            assert!(node1 < node2);
            assert!(node2 > node1);
        }

        #[test]
        fn test_btreeset_deduplication() {
            let mut set = BTreeSet::new();
            let node1 = NodePresence {
                id: 1,
                addr: Ipv4Addr::new(192, 168, 1, 1),
                updated_at: 100,
                is_alive: true,
            };
            let node2 = NodePresence {
                id: 2,
                addr: Ipv4Addr::new(192, 168, 1, 1),
                updated_at: 200,
                is_alive: false,
            };

            set.insert(node1);
            set.replace(node2.clone());

            assert_eq!(set.len(), 1);
            assert_eq!(set.iter().next().unwrap().updated_at, 200);
        }
    }

    mod notify_message_tests {
        use super::*;

        #[tokio::test]
        async fn test_single_node_message() {
            let node = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let mut msg = BytesMut::new();
            msg.put_u128(999); // node_id
            msg.put_u32(Ipv4Addr::new(192, 168, 1, 1).to_bits());
            msg.put_u128(12345);
            msg.put_u64(1000);
            msg.put_u8(1); // is_alive

            node.notify_message(Cow::Borrowed(msg.freeze().as_ref()))
                .await;

            let map = node.map.read().await;
            let nodes = map.get(&12345).unwrap();
            assert_eq!(nodes.len(), 1);
            let node_data = nodes.iter().next().unwrap();
            assert_eq!(node_data.id, 999);
            assert_eq!(node_data.addr, Ipv4Addr::new(192, 168, 1, 1));
            assert_eq!(node_data.updated_at, 1000);
            assert!(node_data.is_alive);

            let event_queue = node.event_queue.read().await;
            assert_eq!(event_queue.get(&12345).unwrap().len(), 1);
        }

        #[tokio::test]
        async fn test_multiple_nodes_message() {
            let node = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let mut msg = BytesMut::new();
            msg.put_u128(999);
            msg.put_u32(Ipv4Addr::new(192, 168, 1, 1).to_bits());
            msg.put_u128(12345);
            msg.put_u64(1000);
            msg.put_u8(1);
            msg.put_u128(888);
            msg.put_u32(Ipv4Addr::new(192, 168, 1, 2).to_bits());
            msg.put_u128(12345);
            msg.put_u64(2000);
            msg.put_u8(0);

            node.notify_message(Cow::Borrowed(msg.freeze().as_ref()))
                .await;

            let map = node.map.read().await;
            let nodes = map.get(&12345).unwrap();
            assert_eq!(nodes.len(), 2);
        }

        #[tokio::test]
        async fn test_update_with_newer_timestamp() {
            let node = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let mut msg1 = BytesMut::new();
            msg1.put_u128(999);
            msg1.put_u32(Ipv4Addr::new(192, 168, 1, 1).to_bits());
            msg1.put_u128(12345);
            msg1.put_u64(1000);
            msg1.put_u8(1);
            node.notify_message(Cow::Borrowed(msg1.freeze().as_ref()))
                .await;

            let mut msg2 = BytesMut::new();
            msg2.put_u128(999);
            msg2.put_u32(Ipv4Addr::new(192, 168, 1, 1).to_bits());
            msg2.put_u128(12345);
            msg2.put_u64(2000);
            msg2.put_u8(0);
            node.notify_message(Cow::Borrowed(msg2.freeze().as_ref()))
                .await;

            let map = node.map.read().await;
            let nodes = map.get(&12345).unwrap();
            assert_eq!(nodes.len(), 1);
            let node_data = nodes.iter().next().unwrap();
            assert_eq!(node_data.updated_at, 2000);
            assert!(!node_data.is_alive);
        }

        #[tokio::test]
        async fn test_ignore_older_timestamp() {
            let node = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let mut msg1 = BytesMut::new();
            msg1.put_u128(999);
            msg1.put_u32(Ipv4Addr::new(192, 168, 1, 1).to_bits());
            msg1.put_u128(12345);
            msg1.put_u64(2000);
            msg1.put_u8(1);
            node.notify_message(Cow::Borrowed(msg1.freeze().as_ref()))
                .await;

            let _event_queue_before = node.event_queue.read().await.len();

            let mut msg2 = BytesMut::new();
            msg2.put_u128(999);
            msg2.put_u32(Ipv4Addr::new(192, 168, 1, 1).to_bits());
            msg2.put_u128(12345);
            msg2.put_u64(1000);
            msg2.put_u8(1);
            node.notify_message(Cow::Borrowed(msg2.freeze().as_ref()))
                .await;

            let map = node.map.read().await;
            let nodes = map.get(&12345).unwrap();
            let node_data = nodes.iter().next().unwrap();
            assert_eq!(node_data.updated_at, 2000);
            assert!(node_data.is_alive);

            let event_queue = node.event_queue.read().await;
            assert_eq!(event_queue.get(&12345).unwrap().len(), 1);
        }

        #[tokio::test]
        async fn test_empty_message() {
            let node = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let msg = BytesMut::new();
            node.notify_message(Cow::Borrowed(msg.freeze().as_ref()))
                .await;

            let map = node.map.read().await;
            assert_eq!(map.len(), 0);
        }

        #[tokio::test]
        async fn test_multiple_code_ids() {
            let node = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let mut msg = BytesMut::new();
            msg.put_u128(999);
            msg.put_u32(Ipv4Addr::new(192, 168, 1, 1).to_bits());
            msg.put_u128(12345);
            msg.put_u64(1000);
            msg.put_u8(1);
            msg.put_u128(888);
            msg.put_u32(Ipv4Addr::new(192, 168, 1, 2).to_bits());
            msg.put_u128(67890);
            msg.put_u64(2000);
            msg.put_u8(1);

            node.notify_message(Cow::Borrowed(msg.freeze().as_ref()))
                .await;

            let map = node.map.read().await;
            assert_eq!(map.len(), 2);
            assert!(map.contains_key(&12345));
            assert!(map.contains_key(&67890));
        }
    }

    mod broadcast_messages_tests {
        use super::*;

        #[tokio::test]
        async fn test_broadcast_single_message() {
            let node = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let mut event_queue = node.event_queue.write().await;
            event_queue.entry(12345).or_default().insert(NodePresence {
                id: 999,
                addr: Ipv4Addr::new(192, 168, 1, 1),
                updated_at: 1000,
                is_alive: true,
            });
            drop(event_queue);

            let messages: Vec<_> = node
                .broadcast_messages(1000, |bytes| {
                    let len = bytes.len();
                    (len, bytes)
                })
                .await
                .collect();

            assert_eq!(messages.len(), 1);
            assert_eq!(messages[0].len(), 45);

            let event_queue = node.event_queue.read().await;
            assert_eq!(event_queue.len(), 0);
        }

        #[tokio::test]
        async fn test_broadcast_respects_size_limit() {
            let node = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let mut event_queue = node.event_queue.write().await;
            for i in 0..10 {
                event_queue.entry(12345).or_default().insert(NodePresence {
                    id: i as u128,
                    addr: Ipv4Addr::new(192, 168, 1, i),
                    updated_at: 1000,
                    is_alive: true,
                });
            }
            drop(event_queue);

            let messages: Vec<_> = node
                .broadcast_messages(100, |bytes| {
                    let len = bytes.len();
                    (len, bytes)
                })
                .await
                .collect();

            assert_eq!(messages.len(), 2);

            let event_queue = node.event_queue.read().await;
            let remaining = event_queue.get(&12345).map(|s| s.len()).unwrap_or(0);
            assert_eq!(remaining, 8);
        }

        #[tokio::test]
        async fn test_broadcast_multiple_code_ids() {
            let node = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let mut event_queue = node.event_queue.write().await;
            event_queue.entry(12345).or_default().insert(NodePresence {
                id: 999,
                addr: Ipv4Addr::new(192, 168, 1, 1),
                updated_at: 1000,
                is_alive: true,
            });
            event_queue.entry(67890).or_default().insert(NodePresence {
                id: 888,
                addr: Ipv4Addr::new(192, 168, 1, 2),
                updated_at: 2000,
                is_alive: false,
            });
            drop(event_queue);

            let messages: Vec<_> = node
                .broadcast_messages(1000, |bytes| {
                    let len = bytes.len();
                    (len, bytes)
                })
                .await
                .collect();

            assert_eq!(messages.len(), 2);

            let event_queue = node.event_queue.read().await;
            assert_eq!(event_queue.len(), 0);
        }

        #[tokio::test]
        async fn test_broadcast_empty_queue() {
            let node = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let messages: Vec<_> = node
                .broadcast_messages(1000, |bytes| {
                    let len = bytes.len();
                    (len, bytes)
                })
                .await
                .collect();

            assert_eq!(messages.len(), 0);
        }
    }

    mod local_state_tests {
        use super::*;

        #[tokio::test]
        async fn test_empty_map() {
            let node = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let state = node.local_state(false).await;
            assert_eq!(state.len(), 0);
        }

        #[tokio::test]
        async fn test_single_node_serialization() {
            let node = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let mut map = node.map.write().await;
            map.entry(12345).or_default().insert(NodePresence {
                id: 999,
                addr: Ipv4Addr::new(192, 168, 1, 1),
                updated_at: 1000,
                is_alive: true,
            });
            drop(map);

            let state = node.local_state(false).await;
            assert_eq!(state.len(), 45);

            let mut reader = state.as_ref();
            assert_eq!(reader.get_u128(), 999);
            assert_eq!(
                Ipv4Addr::from(reader.get_u32()),
                Ipv4Addr::new(192, 168, 1, 1)
            );
            assert_eq!(reader.get_u128(), 12345);
            assert_eq!(reader.get_u64(), 1000);
            assert_eq!(reader.get_u8(), 1);
        }

        #[tokio::test]
        async fn test_multiple_nodes_serialization() {
            let node = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let mut map = node.map.write().await;
            map.entry(12345).or_default().insert(NodePresence {
                id: 999,
                addr: Ipv4Addr::new(192, 168, 1, 1),
                updated_at: 1000,
                is_alive: true,
            });
            map.entry(12345).or_default().insert(NodePresence {
                id: 888,
                addr: Ipv4Addr::new(192, 168, 1, 2),
                updated_at: 2000,
                is_alive: false,
            });
            drop(map);

            let state = node.local_state(false).await;
            assert_eq!(state.len(), 90);
        }

        #[tokio::test]
        async fn test_multiple_code_ids_serialization() {
            let node = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let mut map = node.map.write().await;
            map.entry(12345).or_default().insert(NodePresence {
                id: 999,
                addr: Ipv4Addr::new(192, 168, 1, 1),
                updated_at: 1000,
                is_alive: true,
            });
            map.entry(67890).or_default().insert(NodePresence {
                id: 888,
                addr: Ipv4Addr::new(192, 168, 1, 2),
                updated_at: 2000,
                is_alive: false,
            });
            drop(map);

            let state = node.local_state(false).await;
            assert_eq!(state.len(), 90);
        }
    }

    mod merge_remote_state_tests {
        use super::*;

        #[tokio::test]
        async fn test_merge_remote_state() {
            let node = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let mut msg = BytesMut::new();
            msg.put_u128(999);
            msg.put_u32(Ipv4Addr::new(192, 168, 1, 1).to_bits());
            msg.put_u128(12345);
            msg.put_u64(1000);
            msg.put_u8(1);

            node.merge_remote_state(msg.freeze().as_ref(), false).await;

            let map = node.map.read().await;
            assert_eq!(map.len(), 1);
            assert!(map.contains_key(&12345));
        }
    }

    mod integration_tests {
        use super::*;
        use std::sync::Arc;

        #[tokio::test]
        async fn test_node_lifecycle() {
            let node = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let mut msg1 = BytesMut::new();
            msg1.put_u128(999);
            msg1.put_u32(Ipv4Addr::new(192, 168, 1, 1).to_bits());
            msg1.put_u128(12345);
            msg1.put_u64(1000);
            msg1.put_u8(1);
            node.notify_message(Cow::Borrowed(msg1.freeze().as_ref()))
                .await;

            let mut msg2 = BytesMut::new();
            msg2.put_u128(999);
            msg2.put_u32(Ipv4Addr::new(192, 168, 1, 1).to_bits());
            msg2.put_u128(12345);
            msg2.put_u64(2000);
            msg2.put_u8(0);
            node.notify_message(Cow::Borrowed(msg2.freeze().as_ref()))
                .await;

            let messages: Vec<_> = node
                .broadcast_messages(1000, |bytes| {
                    let len = bytes.len();
                    (len, bytes)
                })
                .await
                .collect();

            assert_eq!(messages.len(), 1);

            let mut reader = messages[0].as_ref();
            assert_eq!(reader.get_u128(), 999);
            assert_eq!(
                Ipv4Addr::from(reader.get_u32()),
                Ipv4Addr::new(192, 168, 1, 1)
            );
            assert_eq!(reader.get_u128(), 12345);
            assert_eq!(reader.get_u64(), 2000);
            assert_eq!(reader.get_u8(), 0);
        }

        #[tokio::test]
        async fn test_full_sync_flow() {
            let node1 = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };
            let node2 = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let mut msg = BytesMut::new();
            msg.put_u128(999);
            msg.put_u32(Ipv4Addr::new(192, 168, 1, 1).to_bits());
            msg.put_u128(12345);
            msg.put_u64(1000);
            msg.put_u8(1);
            node1
                .notify_message(Cow::Borrowed(msg.freeze().as_ref()))
                .await;

            let state = node1.local_state(false).await;
            node2.merge_remote_state(state.as_ref(), false).await;

            let map1 = node1.map.read().await;
            let map2 = node2.map.read().await;
            assert_eq!(map1.len(), map2.len());
            assert_eq!(
                map1.get(&12345).unwrap().len(),
                map2.get(&12345).unwrap().len()
            );
        }

        #[tokio::test]
        async fn test_concurrent_notify_messages() {
            let node = Arc::new(WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            });

            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let node_ref = Arc::clone(&node);
                    tokio::spawn(async move {
                        let mut msg = BytesMut::new();
                        msg.put_u128(i as u128);
                        msg.put_u32(Ipv4Addr::new(192, 168, 1, i).to_bits());
                        msg.put_u128(12345);
                        msg.put_u64(1000);
                        msg.put_u8(1);
                        node_ref
                            .notify_message(Cow::Borrowed(msg.freeze().as_ref()))
                            .await;
                    })
                })
                .collect();

            for handle in handles {
                handle.await.unwrap();
            }

            let map = node.map.read().await;
            assert_eq!(map.get(&12345).unwrap().len(), 10);
        }

        #[tokio::test]
        async fn test_message_format_roundtrip() {
            let node1 = WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            };

            let original_id = 999u128;
            let original_addr = Ipv4Addr::new(192, 168, 1, 1);
            let original_code_id = 12345u128;
            let original_updated_at = 1000u64;
            let original_is_alive = true;

            let mut msg = BytesMut::new();
            msg.put_u128(original_id);
            msg.put_u32(original_addr.to_bits());
            msg.put_u128(original_code_id);
            msg.put_u64(original_updated_at);
            msg.put_u8(if original_is_alive { 1 } else { 0 });

            node1
                .notify_message(Cow::Borrowed(msg.freeze().as_ref()))
                .await;

            let state = node1.local_state(false).await;
            let mut reader = state.as_ref();

            assert_eq!(reader.get_u128(), original_id);
            assert_eq!(Ipv4Addr::from(reader.get_u32()), original_addr);
            assert_eq!(reader.get_u128(), original_code_id);
            assert_eq!(reader.get_u64(), original_updated_at);
            assert_eq!(reader.get_u8(), 1);
        }
    }

    // EventDelegate tests
    mod event_delegate_tests {
        use super::*;
        use std::sync::Arc;

        // Test helper functions
        fn create_test_node() -> WarmUpMapNode {
            WarmUpMapNode {
                map: RwLock::new(BTreeMap::new()),
                event_queue: RwLock::new(BTreeMap::new()),
            }
        }

        fn create_node_state(id: NodeId, addr: Ipv4Addr) -> Arc<NodeState<NodeId, Ipv4Addr>> {
            Arc::new(NodeState {
                id,
                addr,
                meta: Default::default(),
                state: Default::default(),
                protocol_version: Default::default(),
                delegate_version: Default::default(),
            })
        }

        async fn add_node_to_map(node: &WarmUpMapNode, code_id: CodeId, presence: NodePresence) {
            let mut map = node.map.write().await;
            map.entry(code_id).or_default().insert(presence);
        }

        async fn get_total_node_count(node: &WarmUpMapNode) -> usize {
            let map = node.map.read().await;
            map.values().map(|set| set.len()).sum()
        }

        async fn node_exists_in_code(
            node: &WarmUpMapNode,
            code_id: CodeId,
            node_id: NodeId,
        ) -> bool {
            let map = node.map.read().await;
            map.get(&code_id)
                .map(|set| set.iter().any(|n| n.id == node_id))
                .unwrap_or(false)
        }

        async fn get_nodes_with_addr(
            node: &WarmUpMapNode,
            code_id: CodeId,
            addr: Ipv4Addr,
        ) -> Vec<NodePresence> {
            let map = node.map.read().await;
            map.get(&code_id)
                .map(|set| {
                    set.iter()
                        .filter(|n| n.addr == addr)
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        }

        // Phase 1: notify_join tests
        mod notify_join_tests {
            use super::*;

            #[tokio::test]
            async fn test_basic_join() {
                let node = create_test_node();
                let node_state = create_node_state(1, Ipv4Addr::new(192, 168, 1, 1));

                node.notify_join(node_state).await;

                // Should complete without error
                assert_eq!(get_total_node_count(&node).await, 0);
            }

            #[tokio::test]
            async fn test_join_removes_same_addr_different_id() {
                let node = create_test_node();
                let code_id = 12345;
                let addr = Ipv4Addr::new(192, 168, 1, 1);

                // Add a node with ID 1 at this address
                add_node_to_map(
                    &node,
                    code_id,
                    NodePresence {
                        id: 1,
                        addr,
                        updated_at: 1000,
                        is_alive: true,
                    },
                )
                .await;

                // Add another node with different ID/address
                add_node_to_map(
                    &node,
                    code_id,
                    NodePresence {
                        id: 2,
                        addr: Ipv4Addr::new(192, 168, 1, 2),
                        updated_at: 1000,
                        is_alive: true,
                    },
                )
                .await;

                assert_eq!(get_total_node_count(&node).await, 2);

                // Node with ID 999 joins at the same address
                let node_state = create_node_state(999, addr);
                node.notify_join(node_state).await;

                // Node with ID 1 should be removed (same addr, different ID)
                assert!(!node_exists_in_code(&node, code_id, 1).await);
                // Node with ID 2 should still exist (different addr)
                assert!(node_exists_in_code(&node, code_id, 2).await);
                assert_eq!(get_total_node_count(&node).await, 1);
            }

            #[tokio::test]
            async fn test_join_across_multiple_code_ids() {
                let node = create_test_node();
                let addr = Ipv4Addr::new(192, 168, 1, 1);

                // Add nodes with ID 1 at same address across multiple code_ids
                for code_id in [100, 200, 300] {
                    add_node_to_map(
                        &node,
                        code_id,
                        NodePresence {
                            id: 1,
                            addr,
                            updated_at: 1000,
                            is_alive: true,
                        },
                    )
                    .await;
                }

                assert_eq!(get_total_node_count(&node).await, 3);

                // Node with ID 999 joins at the same address
                let node_state = create_node_state(999, addr);
                node.notify_join(node_state).await;

                // All nodes with ID 1 should be removed from all code_ids
                for code_id in [100, 200, 300] {
                    assert!(!node_exists_in_code(&node, code_id, 1).await);
                }
                assert_eq!(get_total_node_count(&node).await, 0);
            }

            #[tokio::test]
            async fn test_join_on_empty_map() {
                let node = create_test_node();
                let node_state = create_node_state(1, Ipv4Addr::new(192, 168, 1, 1));

                node.notify_join(node_state).await;

                assert_eq!(get_total_node_count(&node).await, 0);
            }

            #[tokio::test]
            async fn test_join_same_node_twice() {
                let node = create_test_node();
                let code_id = 12345;
                let addr = Ipv4Addr::new(192, 168, 1, 1);

                // Add the node
                add_node_to_map(
                    &node,
                    code_id,
                    NodePresence {
                        id: 1,
                        addr,
                        updated_at: 1000,
                        is_alive: true,
                    },
                )
                .await;

                let node_state = create_node_state(1, addr);

                // Join twice with same ID/address
                node.notify_join(Arc::clone(&node_state)).await;
                node.notify_join(node_state).await;

                // Node should still exist (same ID, so not removed)
                assert!(node_exists_in_code(&node, code_id, 1).await);
                assert_eq!(get_total_node_count(&node).await, 1);
            }

            #[tokio::test]
            async fn test_join_preserves_different_addr_nodes() {
                let node = create_test_node();
                let code_id = 12345;

                // Add multiple nodes with different addresses
                for i in 1..=5 {
                    add_node_to_map(
                        &node,
                        code_id,
                        NodePresence {
                            id: i as u128,
                            addr: Ipv4Addr::new(192, 168, 1, i),
                            updated_at: 1000,
                            is_alive: true,
                        },
                    )
                    .await;
                }

                assert_eq!(get_total_node_count(&node).await, 5);

                // New node joins at address that doesn't conflict
                let node_state = create_node_state(999, Ipv4Addr::new(192, 168, 1, 100));
                node.notify_join(node_state).await;

                // All 5 nodes should still exist
                assert_eq!(get_total_node_count(&node).await, 5);
            }

            #[tokio::test]
            async fn test_join_removes_only_same_addr_different_id() {
                let node = create_test_node();
                let code_id = 12345;
                let addr = Ipv4Addr::new(192, 168, 1, 1);

                // Add node with ID 1 at target address
                add_node_to_map(
                    &node,
                    code_id,
                    NodePresence {
                        id: 1,
                        addr,
                        updated_at: 1000,
                        is_alive: true,
                    },
                )
                .await;

                // Add node with ID 2 at same address (should not happen in practice, but testing)
                add_node_to_map(
                    &node,
                    code_id,
                    NodePresence {
                        id: 2,
                        addr,
                        updated_at: 1000,
                        is_alive: true,
                    },
                )
                .await;

                assert_eq!(get_total_node_count(&node).await, 1); // BTreeSet dedup by addr

                // Node with ID 3 joins
                let node_state = create_node_state(3, addr);
                node.notify_join(node_state).await;

                let nodes_at_addr = get_nodes_with_addr(&node, code_id, addr).await;
                assert_eq!(nodes_at_addr.len(), 0);
            }
        }

        // Phase 1: notify_leave tests
        mod notify_leave_tests {
            use super::*;

            #[tokio::test]
            async fn test_basic_leave() {
                let node = create_test_node();
                let code_id = 12345;

                // Add a node
                add_node_to_map(
                    &node,
                    code_id,
                    NodePresence {
                        id: 1,
                        addr: Ipv4Addr::new(192, 168, 1, 1),
                        updated_at: 1000,
                        is_alive: true,
                    },
                )
                .await;

                assert!(node_exists_in_code(&node, code_id, 1).await);

                // Node leaves
                let node_state = create_node_state(1, Ipv4Addr::new(192, 168, 1, 1));
                node.notify_leave(node_state).await;

                // Node should be removed
                assert!(!node_exists_in_code(&node, code_id, 1).await);
                assert_eq!(get_total_node_count(&node).await, 0);
            }

            #[tokio::test]
            async fn test_leave_removes_from_all_code_ids() {
                let node = create_test_node();

                // Add same node ID across multiple code_ids
                for code_id in [100, 200, 300] {
                    add_node_to_map(
                        &node,
                        code_id,
                        NodePresence {
                            id: 1,
                            addr: Ipv4Addr::new(192, 168, 1, 1),
                            updated_at: 1000,
                            is_alive: true,
                        },
                    )
                    .await;
                }

                assert_eq!(get_total_node_count(&node).await, 3);

                // Node leaves
                let node_state = create_node_state(1, Ipv4Addr::new(192, 168, 1, 1));
                node.notify_leave(node_state).await;

                // Should be removed from all code_ids
                for code_id in [100, 200, 300] {
                    assert!(!node_exists_in_code(&node, code_id, 1).await);
                }
                assert_eq!(get_total_node_count(&node).await, 0);
            }

            #[tokio::test]
            async fn test_leave_nonexistent_node() {
                let node = create_test_node();

                // Try to remove node that doesn't exist
                let node_state = create_node_state(999, Ipv4Addr::new(192, 168, 1, 1));
                node.notify_leave(node_state).await;

                // Should complete without error
                assert_eq!(get_total_node_count(&node).await, 0);
            }

            #[tokio::test]
            async fn test_leave_partial_existence() {
                let node = create_test_node();

                // Add node to only some code_ids
                add_node_to_map(
                    &node,
                    100,
                    NodePresence {
                        id: 1,
                        addr: Ipv4Addr::new(192, 168, 1, 1),
                        updated_at: 1000,
                        is_alive: true,
                    },
                )
                .await;

                add_node_to_map(
                    &node,
                    200,
                    NodePresence {
                        id: 2,
                        addr: Ipv4Addr::new(192, 168, 1, 2),
                        updated_at: 1000,
                        is_alive: true,
                    },
                )
                .await;

                assert_eq!(get_total_node_count(&node).await, 2);

                // Node 1 leaves
                let node_state = create_node_state(1, Ipv4Addr::new(192, 168, 1, 1));
                node.notify_leave(node_state).await;

                // Only node 1 should be removed
                assert!(!node_exists_in_code(&node, 100, 1).await);
                assert!(node_exists_in_code(&node, 200, 2).await);
                assert_eq!(get_total_node_count(&node).await, 1);
            }

            #[tokio::test]
            async fn test_leave_on_empty_map() {
                let node = create_test_node();

                let node_state = create_node_state(1, Ipv4Addr::new(192, 168, 1, 1));
                node.notify_leave(node_state).await;

                assert_eq!(get_total_node_count(&node).await, 0);
            }

            #[tokio::test]
            async fn test_leave_preserves_other_nodes() {
                let node = create_test_node();
                let code_id = 12345;

                // Add multiple nodes
                for i in 1..=5 {
                    add_node_to_map(
                        &node,
                        code_id,
                        NodePresence {
                            id: i as u128,
                            addr: Ipv4Addr::new(192, 168, 1, i),
                            updated_at: 1000,
                            is_alive: true,
                        },
                    )
                    .await;
                }

                assert_eq!(get_total_node_count(&node).await, 5);

                // Node 3 leaves
                let node_state = create_node_state(3, Ipv4Addr::new(192, 168, 1, 3));
                node.notify_leave(node_state).await;

                // Only node 3 should be removed
                assert!(!node_exists_in_code(&node, code_id, 3).await);
                assert_eq!(get_total_node_count(&node).await, 4);

                // Other nodes should still exist
                for i in [1, 2, 4, 5] {
                    assert!(node_exists_in_code(&node, code_id, i).await);
                }
            }
        }

        // Phase 1: notify_update tests
        mod notify_update_tests {
            use super::*;

            #[tokio::test]
            async fn test_update_is_noop() {
                let node = create_test_node();
                let code_id = 12345;

                // Add a node
                add_node_to_map(
                    &node,
                    code_id,
                    NodePresence {
                        id: 1,
                        addr: Ipv4Addr::new(192, 168, 1, 1),
                        updated_at: 1000,
                        is_alive: true,
                    },
                )
                .await;

                let count_before = get_total_node_count(&node).await;

                // Call notify_update
                let node_state = create_node_state(1, Ipv4Addr::new(192, 168, 1, 1));
                node.notify_update(node_state).await;

                let count_after = get_total_node_count(&node).await;

                // State should be unchanged
                assert_eq!(count_before, count_after);
                assert!(node_exists_in_code(&node, code_id, 1).await);
            }

            #[tokio::test]
            async fn test_update_on_empty_map() {
                let node = create_test_node();

                let node_state = create_node_state(1, Ipv4Addr::new(192, 168, 1, 1));
                node.notify_update(node_state).await;

                // Should complete without error
                assert_eq!(get_total_node_count(&node).await, 0);
            }
        }

        // Phase 2: Concurrency tests
        mod concurrency_tests {
            use super::*;

            #[tokio::test]
            async fn test_concurrent_joins() {
                let node = Arc::new(create_test_node());
                let code_id = 12345;

                // Add some initial nodes
                for i in 1..=10 {
                    add_node_to_map(
                        &node,
                        code_id,
                        NodePresence {
                            id: i as u128,
                            addr: Ipv4Addr::new(192, 168, 1, i),
                            updated_at: 1000,
                            is_alive: true,
                        },
                    )
                    .await;
                }

                // Concurrent joins
                let handles: Vec<_> = (100..110)
                    .map(|i| {
                        let node_ref = Arc::clone(&node);
                        tokio::spawn(async move {
                            let node_state = create_node_state(i as u128, Ipv4Addr::new(192, 168, 2, i));
                            node_ref.notify_join(node_state).await;
                        })
                    })
                    .collect();

                for handle in handles {
                    handle.await.unwrap();
                }

                // All original nodes should still exist
                assert_eq!(get_total_node_count(&node).await, 10);
            }

            #[tokio::test]
            async fn test_concurrent_leaves() {
                let node = Arc::new(create_test_node());
                let code_id = 12345;

                // Add nodes
                for i in 1..=10 {
                    add_node_to_map(
                        &node,
                        code_id,
                        NodePresence {
                            id: i as u128,
                            addr: Ipv4Addr::new(192, 168, 1, i),
                            updated_at: 1000,
                            is_alive: true,
                        },
                    )
                    .await;
                }

                // Concurrent leaves
                let handles: Vec<_> = (1..=10)
                    .map(|i| {
                        let node_ref = Arc::clone(&node);
                        tokio::spawn(async move {
                            let node_state = create_node_state(i as u128, Ipv4Addr::new(192, 168, 1, i));
                            node_ref.notify_leave(node_state).await;
                        })
                    })
                    .collect();

                for handle in handles {
                    handle.await.unwrap();
                }

                // All nodes should be removed
                assert_eq!(get_total_node_count(&node).await, 0);
            }

            #[tokio::test]
            async fn test_concurrent_join_and_leave() {
                let node = Arc::new(create_test_node());
                let code_id = 12345;

                // Add initial nodes
                for i in 1..=20 {
                    add_node_to_map(
                        &node,
                        code_id,
                        NodePresence {
                            id: i as u128,
                            addr: Ipv4Addr::new(192, 168, 1, i),
                            updated_at: 1000,
                            is_alive: true,
                        },
                    )
                    .await;
                }

                let mut handles = vec![];

                // Concurrent joins
                for i in 100..110 {
                    let node_ref = Arc::clone(&node);
                    handles.push(tokio::spawn(async move {
                        let node_state = create_node_state(i as u128, Ipv4Addr::new(192, 168, 2, i));
                        node_ref.notify_join(node_state).await;
                    }));
                }

                // Concurrent leaves
                for i in 1..=10 {
                    let node_ref = Arc::clone(&node);
                    handles.push(tokio::spawn(async move {
                        let node_state = create_node_state(i as u128, Ipv4Addr::new(192, 168, 1, i));
                        node_ref.notify_leave(node_state).await;
                    }));
                }

                for handle in handles {
                    handle.await.unwrap();
                }

                // 20 initial - 10 removed = 10 remaining
                assert_eq!(get_total_node_count(&node).await, 10);
            }

            #[tokio::test]
            async fn test_concurrent_join_and_notify_message() {
                let node = Arc::new(create_test_node());

                let mut handles = vec![];

                // Concurrent joins
                for i in 1..=5 {
                    let node_ref = Arc::clone(&node);
                    handles.push(tokio::spawn(async move {
                        let node_state = create_node_state(i as u128, Ipv4Addr::new(192, 168, 1, i));
                        node_ref.notify_join(node_state).await;
                    }));
                }

                // Concurrent notify_message
                for i in 10..15 {
                    let node_ref = Arc::clone(&node);
                    handles.push(tokio::spawn(async move {
                        let mut msg = BytesMut::new();
                        msg.put_u128(i as u128);
                        msg.put_u32(Ipv4Addr::new(192, 168, 1, i).to_bits());
                        msg.put_u128(12345);
                        msg.put_u64(1000);
                        msg.put_u8(1);
                        node_ref
                            .notify_message(Cow::Borrowed(msg.freeze().as_ref()))
                            .await;
                    }));
                }

                for handle in handles {
                    handle.await.unwrap();
                }

                // Should have 5 nodes from notify_message
                assert!(get_total_node_count(&node).await >= 5);
            }
        }

        // Phase 2: Integration tests
        mod integration_tests {
            use super::*;

            #[tokio::test]
            async fn test_node_full_lifecycle() {
                let node = create_test_node();
                let code_id = 12345;
                let node_id = 999;
                let addr = Ipv4Addr::new(192, 168, 1, 1);

                // Initial state: empty
                assert_eq!(get_total_node_count(&node).await, 0);

                // Node joins
                let node_state = create_node_state(node_id, addr);
                node.notify_join(Arc::clone(&node_state)).await;
                assert_eq!(get_total_node_count(&node).await, 0); // notify_join doesn't add

                // Add via notify_message
                let mut msg = BytesMut::new();
                msg.put_u128(node_id);
                msg.put_u32(addr.to_bits());
                msg.put_u128(code_id);
                msg.put_u64(1000);
                msg.put_u8(1);
                node.notify_message(Cow::Borrowed(msg.freeze().as_ref()))
                    .await;

                assert!(node_exists_in_code(&node, code_id, node_id).await);

                // Node updates (no-op currently)
                node.notify_update(Arc::clone(&node_state)).await;
                assert!(node_exists_in_code(&node, code_id, node_id).await);

                // Node leaves
                node.notify_leave(node_state).await;
                assert!(!node_exists_in_code(&node, code_id, node_id).await);
            }

            #[tokio::test]
            async fn test_node_address_change() {
                let node = create_test_node();
                let code_id = 12345;
                let node_id = 999;
                let old_addr = Ipv4Addr::new(192, 168, 1, 1);
                let new_addr = Ipv4Addr::new(192, 168, 1, 2);

                // Node at old address
                add_node_to_map(
                    &node,
                    code_id,
                    NodePresence {
                        id: node_id,
                        addr: old_addr,
                        updated_at: 1000,
                        is_alive: true,
                    },
                )
                .await;

                assert!(node_exists_in_code(&node, code_id, node_id).await);

                // Node joins at new address (simulating IP change)
                let node_state = create_node_state(node_id, new_addr);
                node.notify_join(node_state).await;

                // Old node should still exist (notify_join only removes different IDs at same addr)
                assert!(node_exists_in_code(&node, code_id, node_id).await);
            }

            #[tokio::test]
            async fn test_network_partition_recovery() {
                let node = create_test_node();
                let code_id = 12345;
                let addr = Ipv4Addr::new(192, 168, 1, 1);

                // Old node at address
                add_node_to_map(
                    &node,
                    code_id,
                    NodePresence {
                        id: 1,
                        addr,
                        updated_at: 1000,
                        is_alive: true,
                    },
                )
                .await;

                // After partition, new node joins at same address
                let node_state = create_node_state(999, addr);
                node.notify_join(node_state).await;

                // Old node should be removed
                assert!(!node_exists_in_code(&node, code_id, 1).await);
            }

            #[tokio::test]
            async fn test_event_delegate_with_node_delegate() {
                let node = create_test_node();
                let code_id = 12345;

                // Add nodes via notify_message
                let mut msg = BytesMut::new();
                for i in 1..=3 {
                    msg.put_u128(i as u128);
                    msg.put_u32(Ipv4Addr::new(192, 168, 1, i).to_bits());
                    msg.put_u128(code_id);
                    msg.put_u64(1000);
                    msg.put_u8(1);
                }
                node.notify_message(Cow::Borrowed(msg.freeze().as_ref()))
                    .await;

                assert_eq!(get_total_node_count(&node).await, 3);

                // Node 1 joins at new address
                let node_state = create_node_state(1, Ipv4Addr::new(192, 168, 2, 1));
                node.notify_join(node_state).await;

                // Original nodes should still exist
                assert_eq!(get_total_node_count(&node).await, 3);

                // Node 2 leaves
                let node_state = create_node_state(2, Ipv4Addr::new(192, 168, 1, 2));
                node.notify_leave(node_state).await;

                assert_eq!(get_total_node_count(&node).await, 2);
                assert!(!node_exists_in_code(&node, code_id, 2).await);

                // Check broadcast includes leave event
                let messages: Vec<_> = node
                    .broadcast_messages(1000, |bytes| {
                        let len = bytes.len();
                        (len, bytes)
                    })
                    .await
                    .collect();

                // Should have events from initial adds and update
                assert!(!messages.is_empty());
            }

            #[tokio::test]
            async fn test_stress_100_nodes() {
                let node = create_test_node();
                let code_id = 12345;

                // Add 100 nodes
                for i in 1..=100 {
                    add_node_to_map(
                        &node,
                        code_id,
                        NodePresence {
                            id: i as u128,
                            addr: Ipv4Addr::new(
                                192,
                                168,
                                ((i / 256) % 256) as u8,
                                (i % 256) as u8,
                            ),
                            updated_at: 1000,
                            is_alive: true,
                        },
                    )
                    .await;
                }

                assert_eq!(get_total_node_count(&node).await, 100);

                // Join 50 nodes (some may conflict)
                for i in 1..=50 {
                    let node_state = create_node_state(
                        (i + 1000) as u128,
                        Ipv4Addr::new(192, 168, ((i / 256) % 256) as u8, (i % 256) as u8),
                    );
                    node.notify_join(node_state).await;
                }

                // Should have removed conflicting nodes
                assert!(get_total_node_count(&node).await <= 100);

                // Leave 30 nodes
                for i in 51..=80 {
                    let node_state = create_node_state(
                        i as u128,
                        Ipv4Addr::new(192, 168, ((i / 256) % 256) as u8, (i % 256) as u8),
                    );
                    node.notify_leave(node_state).await;
                }

                // Should have fewer nodes
                let remaining = get_total_node_count(&node).await;
                assert!(remaining <= 70);
            }

            #[tokio::test]
            async fn test_extreme_values() {
                let node = create_test_node();

                // Max values
                let node_state = create_node_state(u128::MAX, Ipv4Addr::BROADCAST);
                node.notify_join(Arc::clone(&node_state)).await;
                node.notify_update(Arc::clone(&node_state)).await;
                node.notify_leave(node_state).await;

                // Should complete without error
                assert_eq!(get_total_node_count(&node).await, 0);
            }
        }
    }
}
