### phase-e 64w width16 uniform

Files: counters-disabled-09-rep1-phase-e-churn-w64-width16-uniform.jsonl, counters-disabled-09-rep2-phase-e-churn-w64-width16-uniform.jsonl. 32,790 transactions, 16.00 mutations/tx, 14.64 leaf jobs/tx, instrumented tx/s 3,346, 3,204 (not a headline).

#### Allocator

| per transaction | admission | planner | serial_execution | dispatch | lane | worker_thread | collect | catalog | wal_assembly | wal_append | state_install | publication | dirty_tracking | group_other | harness | **total** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| alloc_calls | 70 | 103 | 1 | 15 | 125 | 0 | 7 | 35 | 1 | 1 | 0 |  | 0 | 1 | 166 | **526** |
| free_calls | 53 | 34 | 0 | 8 | 33 | 7 | 2 | 4 | 1 | 1 | 1 | 93 | 46 | 75 | 166 | **526** |
| realloc_calls | 0.07 | 2.12 | 0.00 | 0.22 | 0.02 | 0.05 |  | 2.45 |  | 0.14 |  |  |  | 0.10 | 0.14 | **5.31** |
| alloc_bytes | 4,878 | 10,162 | 1,261 | 2,843 | 83,402 | 886 | 5,369 | 4,044 | 1,219 | 4,302 | 0 |  | 946 | 283 | 7,981 | **127,577** |
| free_bytes | 3,887 | 4,400 | 11 | 6,085 | 7,140 | 765 | 3,990 | 2,922 | 1,172 | 4,700 | 277 | 16,534 | 66,573 | 7,732 | 8,182 | **134,370** |

#### Leaf ownership

| per transaction | planner | serial_execution | lane | collect | publication | **total** |
|---|---|---|---|---|---|---|
| leaf_entry_clones |  | 3 | 225 |  |  | **229** |
| leaf_entry_drops |  | 0 | 16 | 3 | 225 | **245** |
| arc_key_clones |  | 3 | 225 |  |  | **229** |
| arc_value_clones |  | 3 | 225 |  |  | **229** |
| arc_key_drops |  | 0 | 16 | 3 | 225 | **245** |
| arc_value_drops |  |  | 16 | 3 | 225 | **245** |
| payload_arcs_created | 16.0 |  |  |  |  | **16.0** |
| leaf_page_clones |  | 0.2 | 14.9 |  |  | **15.1** |
| leaf_vec_capacity_bytes |  | 167 | 10,806 |  |  | **10,973** |

#### 4 KiB images, dirty map, PageDelta

| per transaction | serial_execution | lane | wal_append | dirty_tracking | **total** |
|---|---|---|---|---|---|
| page_image_copies |  |  | 0.24 | 0.23 | **0.47** |
| page_image_bytes_copied |  |  | 998 | 944 | **1,942** |
| page_image_buffers |  | 16.0 |  |  | **16.0** |
| page_encodes | 0.2 | 16.0 |  |  | **16.2** |
| dirty_page_replaces |  |  |  | 14.9 | **14.9** |
| dirty_page_bytes_copied |  |  |  | 943 | **943** |
| delta_payload_buffers |  | 32.0 | 0.5 |  | **32.5** |
| delta_payload_bytes |  | 1,703 | 26 |  | **1,729** |
| delta_verify_images |  |  | 0.24 |  | **0.24** |

#### Planner and jobs

| per transaction | planner | dispatch | **total** |
|---|---|---|---|
| planner_map_inserts | 16.0 |  | **16.0** |
| planner_key_copies | 32.0 |  | **32.0** |
| jobs_built |  | 14.9 | **14.9** |

### phase-e 64w width1 uniform

Files: counters-disabled-06-rep1-phase-e-churn-w64-width1-uniform.jsonl, counters-disabled-06-rep2-phase-e-churn-w64-width1-uniform.jsonl. 423,908 transactions, 1.00 mutations/tx, 0.99 leaf jobs/tx, instrumented tx/s 42,958, 41,659 (not a headline).

#### Allocator

| per transaction | admission | planner | serial_execution | dispatch | lane | worker_thread | collect | catalog | wal_assembly | wal_append | state_install | publication | dirty_tracking | group_other | harness | **total** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| alloc_calls | 6.2 | 10.2 | 0.0 | 1.5 | 8.0 | 0.2 | 3.2 | 3.9 | 1.0 | 0.2 |  |  | 0.0 | 0.1 | 16.0 | **50.5** |
| free_calls | 4.1 | 4.1 | 0.0 | 1.0 | 2.0 | 0.6 | 1.0 | 1.1 | 1.0 | 0.2 | 0.1 | 6.8 | 4.0 | 8.4 | 16.1 | **50.5** |
| realloc_calls | 0.07 | 0.00 |  | 0.15 |  | 0.05 |  | 0.00 |  | 0.07 |  |  |  | 0.07 | 0.10 | **0.51** |
| alloc_bytes | 826 | 1,075 | 20 | 246 | 5,253 | 94 | 635 | 691 | 152 | 349 |  |  | 13 | 14 | 846 | **10,213** |
| free_bytes | 684 | 311 | 0 | 433 | 176 | 49 | 190 | 243 | 112 | 375 | 24 | 1,467 | 4,472 | 1,190 | 1,046 | **10,771** |

#### Leaf ownership

| per transaction | planner | serial_execution | lane | publication | **total** |
|---|---|---|---|---|---|
| leaf_entry_clones |  | 0.0 | 15.1 |  | **15.1** |
| leaf_entry_drops |  | 0.0 | 1.0 | 15.1 | **16.1** |
| arc_key_clones |  | 0.0 | 15.1 |  | **15.1** |
| arc_value_clones |  | 0.0 | 15.1 |  | **15.1** |
| arc_key_drops |  | 0.0 | 1.0 | 15.1 | **16.1** |
| arc_value_drops |  |  | 1.0 | 15.1 | **16.1** |
| payload_arcs_created | 1.00 |  |  |  | **1.00** |
| leaf_page_clones |  | 0.00 | 0.99 |  | **1.00** |
| leaf_vec_capacity_bytes |  | 2 | 723 |  | **725** |

#### 4 KiB images, dirty map, PageDelta

| per transaction | lane | wal_append | dirty_tracking | **total** |
|---|---|---|---|---|
| page_image_copies |  | 0.00 | 0.00 | **0.01** |
| page_image_bytes_copied |  | 12.8 | 12.8 | **25.6** |
| page_image_buffers | 1.00 |  |  | **1.00** |
| page_encodes | 1.00 |  |  | **1.00** |
| dirty_page_replaces |  |  | 1.00 | **1.00** |
| dirty_page_bytes_copied |  |  | 12.8 | **12.8** |
| delta_payload_buffers | 1.99 | 0.01 |  | **2.00** |
| delta_payload_bytes | 106 | 0 |  | **107** |

#### Planner and jobs

| per transaction | planner | dispatch | **total** |
|---|---|---|---|
| planner_map_inserts | 1.00 |  | **1.00** |
| planner_key_copies | 2.00 |  | **2.00** |
| jobs_built |  | 0.99 | **0.99** |

