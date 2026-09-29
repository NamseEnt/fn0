### f1 64w width16 uniform

Files: counters-disabled-09-rep1-f1-churn-w64-width16-uniform.jsonl, counters-disabled-09-rep2-f1-churn-w64-width16-uniform.jsonl. 41,290 transactions, 16.00 mutations/tx, 14.70 leaf jobs/tx, instrumented tx/s 4,054, 4,195 (not a headline).

#### Allocator

| per transaction | admission | planner | serial_execution | dispatch | lane | worker_thread | collect | catalog | wal_assembly | wal_append | state_install | publication | dirty_tracking | group_other | harness | **total** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| alloc_calls | 70 | 103 | 1 | 15 | 123 | 0 | 7 | 35 | 1 | 1 |  |  | 0 | 1 | 166 | **524** |
| free_calls | 53 | 34 | 0 | 8 | 33 | 8 | 2 | 4 | 1 | 1 | 1 | 76 | 46 | 91 | 166 | **524** |
| realloc_calls | 0.07 | 2.12 | 0.00 | 0.22 | 0.02 | 0.05 |  | 2.45 |  | 0.14 |  |  |  | 0.10 | 0.13 | **5.30** |
| alloc_bytes | 4,880 | 10,166 | 1,101 | 2,847 | 105,568 | 908 | 5,969 | 4,261 | 1,222 | 4,274 |  |  | 646 | 283 | 7,981 | **150,107** |
| free_bytes | 3,889 | 4,405 | 8 | 6,429 | 7,188 | 774 | 4,101 | 3,273 | 1,178 | 4,670 | 277 | 37,646 | 66,782 | 8,681 | 8,184 | **157,486** |

#### Leaf ownership

| per transaction | planner | serial_execution | lane | **total** |
|---|---|---|---|---|
| payload_arcs_created | 16.0 |  |  | **16.0** |
| leaf_page_clones |  | 0.2 | 14.9 | **15.0** |

#### 4 KiB images, dirty map, PageDelta

| per transaction | serial_execution | lane | wal_append | dirty_tracking | **total** |
|---|---|---|---|---|---|
| page_image_copies |  |  | 0.17 | 0.16 | **0.32** |
| page_image_bytes_copied |  |  | 682 | 644 | **1,326** |
| page_image_buffers |  | 16.0 |  |  | **16.0** |
| page_encodes | 0.2 | 16.0 |  |  | **16.2** |
| dirty_page_replaces |  |  |  | 14.9 | **14.9** |
| dirty_page_bytes_copied |  |  |  | 643 | **643** |
| delta_payload_buffers |  | 32.0 | 0.3 |  | **32.3** |
| delta_payload_bytes |  | 1,704 | 18 |  | **1,722** |
| delta_verify_images |  |  | 0.17 |  | **0.17** |

#### Planner and jobs

| per transaction | planner | serial_execution | dispatch | lane | **total** |
|---|---|---|---|---|---|
| planner_map_inserts | 16.0 |  |  |  | **16.0** |
| planner_key_copies | 32.0 |  |  |  | **32.0** |
| jobs_built |  |  | 14.9 |  | **14.9** |
| leaf_entries_copied |  | 2 |  | 225 | **227** |
| leaf_slot_bytes_copied |  | 95 |  | 8,995 | **9,090** |
| leaf_payload_bytes_copied |  | 230 |  | 21,738 | **21,967** |
| leaf_key_comparisons |  | 0.9 |  | 80.3 | **81.1** |

### f1 64w width1 uniform

Files: counters-disabled-06-rep1-f1-churn-w64-width1-uniform.jsonl, counters-disabled-06-rep2-f1-churn-w64-width1-uniform.jsonl. 506,651 transactions, 1.00 mutations/tx, 0.99 leaf jobs/tx, instrumented tx/s 49,849, 51,422 (not a headline).

#### Allocator

| per transaction | admission | planner | serial_execution | dispatch | lane | worker_thread | collect | catalog | wal_assembly | wal_append | state_install | publication | dirty_tracking | group_other | harness | **total** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| alloc_calls | 6.2 | 10.2 | 0.1 | 1.5 | 7.9 | 0.2 | 3.2 | 3.9 | 1.1 | 0.2 |  |  | 0.0 | 0.1 | 16.1 | **50.8** |
| free_calls | 4.1 | 4.1 | 0.0 | 0.9 | 2.0 | 0.7 | 1.0 | 1.1 | 1.0 | 0.2 | 0.1 | 5.9 | 4.0 | 9.4 | 16.1 | **50.8** |
| realloc_calls | 0.07 | 0.00 |  | 0.15 | 0.00 | 0.05 |  | 0.00 |  | 0.07 |  |  |  | 0.07 | 0.10 | **0.52** |
| alloc_bytes | 831 | 1,070 | 77 | 244 | 6,697 | 99 | 672 | 721 | 153 | 347 |  |  | 39 | 15 | 846 | **11,811** |
| free_bytes | 688 | 306 | 0 | 445 | 174 | 52 | 213 | 279 | 111 | 373 | 25 | 2,906 | 4,486 | 1,302 | 1,042 | **12,403** |

#### Leaf ownership

| per transaction | planner | serial_execution | lane | **total** |
|---|---|---|---|---|
| payload_arcs_created | 1.00 |  |  | **1.00** |
| leaf_page_clones |  | 0.01 | 0.99 | **1.00** |

#### 4 KiB images, dirty map, PageDelta

| per transaction | serial_execution | lane | wal_append | dirty_tracking | **total** |
|---|---|---|---|---|---|
| page_image_copies |  |  | 0.01 | 0.01 | **0.02** |
| page_image_bytes_copied |  |  | 39.3 | 39.3 | **78.5** |
| page_image_buffers |  | 0.99 |  |  | **0.99** |
| page_encodes | 0.01 | 0.99 |  |  | **1.00** |
| dirty_page_replaces |  |  |  | 1.00 | **1.00** |
| dirty_page_bytes_copied |  |  |  | 39.3 | **39.3** |
| delta_payload_buffers |  | 1.98 | 0.02 |  | **2.00** |
| delta_payload_bytes |  | 106 | 1 |  | **107** |
| delta_verify_images |  |  | 0.01 |  | **0.01** |

#### Planner and jobs

| per transaction | planner | serial_execution | dispatch | lane | **total** |
|---|---|---|---|---|---|
| planner_map_inserts | 1.00 |  |  |  | **1.00** |
| planner_key_copies | 2.00 |  |  |  | **2.00** |
| jobs_built |  |  | 0.99 |  | **0.99** |
| leaf_entries_copied |  | 0.1 |  | 15.0 | **15.1** |
| leaf_slot_bytes_copied |  | 6 |  | 598 | **604** |
| leaf_payload_bytes_copied |  | 14 |  | 1,446 | **1,460** |
| leaf_key_comparisons |  | 0.05 |  | 4.97 | **5.02** |

