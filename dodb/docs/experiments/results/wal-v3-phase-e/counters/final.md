### final 64w width16 uniform

Files: counters-disabled-09-rep1-final-churn-w64-width16-uniform.jsonl, counters-disabled-09-rep2-final-churn-w64-width16-uniform.jsonl. 31,612 transactions, 16.00 mutations/tx, 14.66 leaf jobs/tx, instrumented tx/s 3,102, 3,204 (not a headline).

#### Allocator

| per transaction | admission | planner | serial_execution | dispatch | lane | worker_thread | collect | catalog | wal_assembly | wal_append | state_install | publication | dirty_tracking | group_other | harness | **total** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| alloc_calls | 70 | 103 | 1 | 15 | 125 | 0 | 7 | 35 | 1 | 1 | 0 |  | 0 | 1 | 166 | **526** |
| free_calls | 53 | 34 | 0 | 8 | 33 | 7 | 2 | 4 | 1 | 1 | 1 | 93 | 46 | 75 | 166 | **526** |
| realloc_calls | 0.07 | 2.12 | 0.00 | 0.23 | 0.02 | 0.06 |  | 2.44 |  | 0.14 |  |  |  | 0.10 | 0.13 | **5.30** |
| alloc_bytes | 4,879 | 10,163 | 1,148 | 2,843 | 83,405 | 889 | 5,380 | 4,057 | 1,220 | 4,288 | 0 |  | 863 | 283 | 7,982 | **127,399** |
| free_bytes | 3,888 | 4,401 | 10 | 6,103 | 7,133 | 767 | 3,890 | 2,929 | 1,174 | 4,688 | 277 | 16,547 | 66,588 | 7,639 | 8,167 | **134,201** |

#### Leaf ownership

| per transaction | planner | serial_execution | lane | collect | publication | **total** |
|---|---|---|---|---|---|---|
| leaf_entry_clones |  | 3 | 225 |  |  | **228** |
| leaf_entry_drops |  | 0 | 16 | 3 | 225 | **245** |
| arc_key_clones |  | 3 | 225 |  |  | **228** |
| arc_value_clones |  | 3 | 225 |  |  | **228** |
| arc_key_drops |  | 0 | 16 | 3 | 225 | **245** |
| arc_value_drops |  |  | 16 | 3 | 225 | **244** |
| payload_arcs_created | 16.0 |  |  |  |  | **16.0** |
| leaf_page_clones |  | 0.2 | 14.9 |  |  | **15.1** |
| leaf_vec_capacity_bytes |  | 152 | 10,808 |  |  | **10,960** |

#### 4 KiB images, dirty map, PageDelta

| per transaction | serial_execution | lane | wal_append | dirty_tracking | **total** |
|---|---|---|---|---|---|
| page_image_copies |  |  | 0.22 | 0.21 | **0.43** |
| page_image_bytes_copied |  |  | 908 | 860 | **1,768** |
| page_image_buffers |  | 16.0 |  |  | **16.0** |
| page_encodes | 0.2 | 16.0 |  |  | **16.2** |
| dirty_page_replaces |  |  |  | 14.9 | **14.9** |
| dirty_page_bytes_copied |  |  |  | 860 | **860** |
| delta_payload_buffers |  | 32.0 | 0.4 |  | **32.4** |
| delta_payload_bytes |  | 1,704 | 24 |  | **1,727** |
| delta_verify_images |  |  | 0.22 |  | **0.22** |

#### Planner and jobs

| per transaction | planner | dispatch | **total** |
|---|---|---|---|
| planner_map_inserts | 16.0 |  | **16.0** |
| planner_key_copies | 32.0 |  | **32.0** |
| jobs_built |  | 14.9 | **14.9** |

### final 64w width1 uniform

Files: counters-disabled-06-rep1-final-churn-w64-width1-uniform.jsonl, counters-disabled-06-rep2-final-churn-w64-width1-uniform.jsonl. 405,223 transactions, 1.00 mutations/tx, 0.99 leaf jobs/tx, instrumented tx/s 40,622, 40,169 (not a headline).

#### Allocator

| per transaction | admission | planner | serial_execution | dispatch | lane | worker_thread | collect | catalog | wal_assembly | wal_append | state_install | publication | dirty_tracking | group_other | harness | **total** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| alloc_calls | 6.2 | 10.2 | 0.0 | 1.5 | 8.0 | 0.2 | 3.2 | 3.9 | 1.0 | 0.2 |  |  | 0.0 | 0.1 | 16.0 | **50.6** |
| free_calls | 4.1 | 4.1 | 0.0 | 1.0 | 2.0 | 0.7 | 1.0 | 1.1 | 1.0 | 0.2 | 0.1 | 6.8 | 4.0 | 8.4 | 16.1 | **50.6** |
| realloc_calls | 0.07 | 0.00 |  | 0.15 |  | 0.05 |  | 0.00 |  | 0.07 |  |  |  | 0.07 | 0.10 | **0.53** |
| alloc_bytes | 827 | 1,070 | 23 | 246 | 5,250 | 96 | 636 | 696 | 152 | 345 |  |  | 15 | 15 | 846 | **10,218** |
| free_bytes | 685 | 306 | 0 | 434 | 176 | 50 | 194 | 246 | 112 | 371 | 24 | 1,471 | 4,472 | 1,194 | 1,050 | **10,785** |

#### Leaf ownership

| per transaction | planner | serial_execution | lane | publication | **total** |
|---|---|---|---|---|---|
| leaf_entry_clones |  | 0.1 | 15.0 |  | **15.1** |
| leaf_entry_drops |  | 0.0 | 1.0 | 15.1 | **16.1** |
| arc_key_clones |  | 0.1 | 15.0 |  | **15.1** |
| arc_value_clones |  | 0.1 | 15.0 |  | **15.1** |
| arc_key_drops |  | 0.0 | 1.0 | 15.1 | **16.1** |
| arc_value_drops |  |  | 1.0 | 15.1 | **16.1** |
| payload_arcs_created | 1.00 |  |  |  | **1.00** |
| leaf_page_clones |  | 0.00 | 0.99 |  | **1.00** |
| leaf_vec_capacity_bytes |  | 3 | 722 |  | **725** |

#### 4 KiB images, dirty map, PageDelta

| per transaction | lane | wal_append | dirty_tracking | **total** |
|---|---|---|---|---|
| page_image_copies |  | 0.00 | 0.00 | **0.01** |
| page_image_bytes_copied |  | 15.1 | 15.1 | **30.2** |
| page_image_buffers | 1.00 |  |  | **1.00** |
| page_encodes | 1.00 |  |  | **1.00** |
| dirty_page_replaces |  |  | 1.00 | **1.00** |
| dirty_page_bytes_copied |  |  | 15.1 | **15.1** |
| delta_payload_buffers | 1.99 | 0.01 |  | **2.00** |
| delta_payload_bytes | 106 | 0 |  | **107** |

#### Planner and jobs

| per transaction | planner | dispatch | **total** |
|---|---|---|---|
| planner_map_inserts | 1.00 |  | **1.00** |
| planner_key_copies | 2.00 |  | **2.00** |
| jobs_built |  | 0.99 | **0.99** |

