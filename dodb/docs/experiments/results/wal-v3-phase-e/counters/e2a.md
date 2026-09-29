### e2a 64w width16 uniform

Files: counters-disabled-09-rep1-e2a-churn-w64-width16-uniform.jsonl, counters-disabled-09-rep2-e2a-churn-w64-width16-uniform.jsonl. 27,065 transactions, 16.00 mutations/tx, 14.73 leaf jobs/tx, instrumented tx/s 2,693, 2,703 (not a headline).

#### Allocator

| per transaction | admission | planner | serial_execution | dispatch | lane | worker_thread | collect | catalog | wal_assembly | wal_append | state_install | publication | dirty_tracking | group_other | harness | **total** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| alloc_calls | 86 | 162 | 1 | 15 | 141 | 0 | 7 | 35 | 1 | 1 | 0 |  | 0 | 1 | 166 | **617** |
| free_calls | 69 | 61 | 0 | 8 | 33 | 7 | 2 | 4 | 1 | 1 | 1 | 93 | 46 | 123 | 166 | **617** |
| realloc_calls | 16.1 | 0.3 | 0.0 | 0.2 | 0.0 | 0.1 |  | 2.5 |  | 0.1 |  |  |  | 0.1 | 0.1 | **19.5** |
| alloc_bytes | 6,271 | 11,350 | 782 | 2,852 | 84,685 | 877 | 5,391 | 4,037 | 1,223 | 4,271 | 0 |  | 574 | 284 | 7,982 | **130,580** |
| free_bytes | 5,482 | 5,283 | 7 | 6,100 | 7,141 | 755 | 3,524 | 2,925 | 1,179 | 4,667 | 277 | 16,528 | 66,611 | 9,646 | 8,173 | **138,301** |

#### Leaf ownership

| per transaction | serial_execution | lane | collect | publication | **total** |
|---|---|---|---|---|---|
| leaf_entry_clones | 2 | 225 |  |  | **227** |
| leaf_entry_drops | 0 | 16 | 2 | 225 | **243** |
| arc_key_clones | 2 | 225 |  |  | **227** |
| arc_value_clones | 2 | 225 |  |  | **227** |
| arc_key_drops | 0 | 16 | 2 | 225 | **243** |
| arc_value_drops |  | 16 | 2 | 225 | **243** |
| leaf_page_clones | 0.1 | 14.9 |  |  | **15.0** |
| leaf_vec_capacity_bytes | 101 | 10,808 |  |  | **10,909** |

#### 4 KiB images, dirty map, PageDelta

| per transaction | serial_execution | lane | wal_append | dirty_tracking | **total** |
|---|---|---|---|---|---|
| page_image_copies |  |  | 0.15 | 0.14 | **0.29** |
| page_image_bytes_copied |  |  | 609 | 573 | **1,182** |
| page_image_buffers |  | 16.0 |  |  | **16.0** |
| page_encodes | 0.2 | 16.0 |  |  | **16.1** |
| dirty_page_replaces |  |  |  | 14.9 | **14.9** |
| dirty_page_bytes_copied |  |  |  | 572 | **572** |
| delta_payload_buffers |  | 32.0 | 0.3 |  | **32.3** |
| delta_payload_bytes |  | 1,703 | 16 |  | **1,719** |
| delta_verify_images |  |  | 0.15 |  | **0.15** |

#### Planner and jobs

| per transaction | planner | dispatch | **total** |
|---|---|---|---|
| planner_map_inserts | 1,250 |  | **1,250** |
| planner_key_copies | 48.0 |  | **48.0** |
| planner_mutation_clones | 16.0 |  | **16.0** |
| jobs_built |  | 14.9 | **14.9** |

### e2a 64w width1 uniform

Files: counters-disabled-06-rep1-e2a-churn-w64-width1-uniform.jsonl, counters-disabled-06-rep2-e2a-churn-w64-width1-uniform.jsonl. 417,956 transactions, 1.00 mutations/tx, 0.99 leaf jobs/tx, instrumented tx/s 41,375, 42,072 (not a headline).

#### Allocator

| per transaction | admission | planner | serial_execution | dispatch | lane | worker_thread | collect | catalog | wal_assembly | wal_append | state_install | publication | dirty_tracking | group_other | harness | **total** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| alloc_calls | 7.2 | 13.7 | 0.0 | 1.5 | 9.0 | 0.2 | 3.2 | 3.9 | 1.0 | 0.2 |  |  | 0.0 | 0.1 | 16.0 | **56.0** |
| free_calls | 5.1 | 5.6 | 0.0 | 0.9 | 2.0 | 0.6 | 1.0 | 1.1 | 1.0 | 0.2 | 0.1 | 6.8 | 4.0 | 11.4 | 16.1 | **56.0** |
| realloc_calls | 1.07 | 0.14 |  | 0.15 |  | 0.05 |  | 0.00 |  | 0.07 |  |  |  | 0.07 | 0.10 | **1.65** |
| alloc_bytes | 914 | 1,073 | 23 | 244 | 5,330 | 93 | 635 | 688 | 152 | 353 |  |  | 15 | 14 | 846 | **10,382** |
| free_bytes | 785 | 458 | 0 | 431 | 177 | 50 | 187 | 243 | 112 | 378 | 24 | 1,465 | 4,471 | 1,364 | 1,045 | **11,190** |

#### Leaf ownership

| per transaction | serial_execution | lane | publication | **total** |
|---|---|---|---|---|
| leaf_entry_clones | 0.1 | 15.0 |  | **15.1** |
| leaf_entry_drops | 0.0 | 1.0 | 15.1 | **16.1** |
| arc_key_clones | 0.1 | 15.0 |  | **15.1** |
| arc_value_clones | 0.1 | 15.0 |  | **15.1** |
| arc_key_drops | 0.0 | 1.0 | 15.1 | **16.1** |
| arc_value_drops |  | 1.0 | 15.1 | **16.1** |
| leaf_page_clones | 0.00 | 0.99 |  | **1.00** |
| leaf_vec_capacity_bytes | 3 | 722 |  | **725** |

#### 4 KiB images, dirty map, PageDelta

| per transaction | lane | wal_append | dirty_tracking | **total** |
|---|---|---|---|---|
| page_image_copies |  | 0.00 | 0.00 | **0.01** |
| page_image_bytes_copied |  | 14.7 | 14.7 | **29.4** |
| page_image_buffers | 1.00 |  |  | **1.00** |
| page_encodes | 1.00 |  |  | **1.00** |
| dirty_page_replaces |  |  | 1.00 | **1.00** |
| dirty_page_bytes_copied |  |  | 14.7 | **14.7** |
| delta_payload_buffers | 1.99 | 0.01 |  | **2.00** |
| delta_payload_bytes | 106 | 0 |  | **107** |

#### Planner and jobs

| per transaction | planner | dispatch | **total** |
|---|---|---|---|
| planner_map_inserts | 5.00 |  | **5.00** |
| planner_key_copies | 3.00 |  | **3.00** |
| planner_mutation_clones | 1.00 |  | **1.00** |
| jobs_built |  | 0.99 | **0.99** |

