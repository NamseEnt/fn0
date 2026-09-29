### e2b 64w width16 uniform

Files: counters-disabled-09-rep1-e2b-churn-w64-width16-uniform.jsonl, counters-disabled-09-rep2-e2b-churn-w64-width16-uniform.jsonl. 28,827 transactions, 16.00 mutations/tx, 14.69 leaf jobs/tx, instrumented tx/s 2,779, 2,972 (not a headline).

#### Allocator

| per transaction | admission | planner | serial_execution | dispatch | lane | worker_thread | collect | catalog | wal_assembly | wal_append | state_install | publication | dirty_tracking | group_other | harness | **total** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| alloc_calls | 86 | 162 | 1 | 15 | 125 | 0 | 7 | 35 | 1 | 1 | 0 |  | 0 | 1 | 166 | **601** |
| free_calls | 69 | 61 | 0 | 8 | 33 | 8 | 2 | 4 | 1 | 1 | 1 | 77 | 46 | 123 | 166 | **601** |
| realloc_calls | 16.1 | 0.3 | 0.0 | 0.2 | 0.0 | 0.1 |  | 2.5 |  | 0.1 |  |  |  | 0.1 | 0.1 | **19.5** |
| alloc_bytes | 6,265 | 11,347 | 1,114 | 2,837 | 94,634 | 892 | 5,382 | 4,034 | 1,221 | 4,299 | 0 |  | 736 | 283 | 7,982 | **141,027** |
| free_bytes | 5,477 | 5,280 | 9 | 6,097 | 7,151 | 772 | 3,831 | 2,924 | 1,176 | 4,693 | 277 | 26,480 | 66,583 | 9,808 | 8,170 | **148,728** |

#### Leaf ownership

| per transaction | serial_execution | lane | collect | publication | **total** |
|---|---|---|---|---|---|
| leaf_entry_clones | 3 | 225 |  |  | **228** |
| leaf_entry_drops | 0 | 16 | 3 | 225 | **244** |
| arc_value_clones | 3 | 225 |  |  | **228** |
| arc_value_drops |  | 16 | 3 | 225 | **244** |
| leaf_page_clones | 0.2 | 14.9 |  |  | **15.0** |
| leaf_vec_capacity_bytes | 260 | 21,609 |  |  | **21,869** |

#### 4 KiB images, dirty map, PageDelta

| per transaction | serial_execution | lane | wal_append | dirty_tracking | **total** |
|---|---|---|---|---|---|
| page_image_copies |  |  | 0.19 | 0.18 | **0.37** |
| page_image_bytes_copied |  |  | 772 | 734 | **1,506** |
| page_image_buffers |  | 16.0 |  |  | **16.0** |
| page_encodes | 0.2 | 16.0 |  |  | **16.2** |
| dirty_page_replaces |  |  |  | 14.9 | **14.9** |
| dirty_page_bytes_copied |  |  |  | 733 | **733** |
| delta_payload_buffers |  | 32.0 | 0.4 |  | **32.3** |
| delta_payload_bytes |  | 1,703 | 20 |  | **1,723** |
| delta_verify_images |  |  | 0.19 |  | **0.19** |

#### Planner and jobs

| per transaction | planner | dispatch | **total** |
|---|---|---|---|
| planner_map_inserts | 1,254 |  | **1,254** |
| planner_key_copies | 48.0 |  | **48.0** |
| planner_mutation_clones | 16.0 |  | **16.0** |
| jobs_built |  | 14.9 | **14.9** |

### e2b 64w width1 uniform

Files: counters-disabled-06-rep1-e2b-churn-w64-width1-uniform.jsonl, counters-disabled-06-rep2-e2b-churn-w64-width1-uniform.jsonl. 449,896 transactions, 1.00 mutations/tx, 0.99 leaf jobs/tx, instrumented tx/s 44,381, 45,431 (not a headline).

#### Allocator

| per transaction | admission | planner | serial_execution | dispatch | lane | worker_thread | collect | catalog | wal_assembly | wal_append | state_install | publication | dirty_tracking | group_other | harness | **total** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| alloc_calls | 7.2 | 13.7 | 0.1 | 1.5 | 7.9 | 0.2 | 3.2 | 3.9 | 1.1 | 0.2 |  |  | 0.0 | 0.1 | 16.1 | **55.1** |
| free_calls | 5.1 | 5.6 | 0.0 | 0.9 | 2.0 | 0.6 | 1.0 | 1.1 | 1.0 | 0.2 | 0.1 | 5.8 | 4.0 | 11.4 | 16.1 | **55.1** |
| realloc_calls | 1.07 | 0.14 |  | 0.14 |  | 0.05 |  | 0.00 |  | 0.07 |  |  |  | 0.07 | 0.09 | **1.64** |
| alloc_bytes | 916 | 1,078 | 46 | 243 | 5,981 | 93 | 632 | 692 | 153 | 357 |  |  | 27 | 15 | 846 | **11,079** |
| free_bytes | 787 | 460 | 0 | 435 | 176 | 50 | 183 | 246 | 111 | 381 | 24 | 2,138 | 4,470 | 1,373 | 1,041 | **11,878** |

#### Leaf ownership

| per transaction | serial_execution | lane | publication | **total** |
|---|---|---|---|---|
| leaf_entry_clones | 0.1 | 15.0 |  | **15.1** |
| leaf_entry_drops | 0.0 | 1.0 | 15.1 | **16.1** |
| arc_value_clones | 0.1 | 15.0 |  | **15.1** |
| arc_value_drops |  | 1.0 | 15.1 | **16.1** |
| leaf_page_clones | 0.01 | 0.99 |  | **1.00** |
| leaf_vec_capacity_bytes | 9 | 1,440 |  | **1,449** |

#### 4 KiB images, dirty map, PageDelta

| per transaction | serial_execution | lane | wal_append | dirty_tracking | **total** |
|---|---|---|---|---|---|
| page_image_copies |  |  | 0.01 | 0.01 | **0.01** |
| page_image_bytes_copied |  |  | 26.6 | 26.6 | **53.3** |
| page_image_buffers |  | 0.99 |  |  | **0.99** |
| page_encodes | 0.01 | 0.99 |  |  | **1.00** |
| dirty_page_replaces |  |  |  | 1.00 | **1.00** |
| dirty_page_bytes_copied |  |  |  | 26.6 | **26.6** |
| delta_payload_buffers |  | 1.99 | 0.01 |  | **2.00** |
| delta_payload_bytes |  | 106 | 1 |  | **107** |
| delta_verify_images |  |  | 0.01 |  | **0.01** |

#### Planner and jobs

| per transaction | planner | dispatch | **total** |
|---|---|---|---|
| planner_map_inserts | 5.00 |  | **5.00** |
| planner_key_copies | 3.00 |  | **3.00** |
| planner_mutation_clones | 1.00 |  | **1.00** |
| jobs_built |  | 0.99 | **0.99** |

