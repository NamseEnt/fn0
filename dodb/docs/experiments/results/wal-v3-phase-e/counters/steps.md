### 64w width16 uniform (per committed transaction; allocator rows exclude the benchmark harness)

| counter | e0 | e1 | e2a | e2b | e3 | final |
|---|---|---|---|---|---|---|
| alloc_calls | 477.1 | 478.7 | 450.5 | 434.6 | 344.0 | 360.3 |
| free_calls | 477.1 | 478.7 | 450.5 | 434.6 | 343.9 | 360.3 |
| alloc_bytes | 129,279.1 | 133,219.0 | 122,597.5 | 133,045.3 | 129,717.1 | 119,417.2 |
| leaf_entry_clones | 451.9 | 452.3 | 227.3 | 227.8 | 228.4 | 228.3 |
| leaf_entry_drops | 468.1 | 468.5 | 243.4 | 244.0 | 244.7 | 244.6 |
| arc_key_clones | 451.9 | 452.3 | 227.3 | 0.0 | 0.0 | 228.3 |
| arc_value_clones | 451.9 | 452.3 | 227.3 | 227.8 | 228.4 | 228.3 |
| arc_key_drops | 468.1 | 468.5 | 243.4 | 0.0 | 0.0 | 244.6 |
| arc_value_drops | 467.9 | 468.3 | 243.3 | 243.8 | 244.4 | 244.3 |
| leaf_page_clones | 29.8 | 29.9 | 15.0 | 15.0 | 15.1 | 15.1 |
| leaf_vec_capacity_bytes | 21,692.2 | 21,711.9 | 10,909.1 | 21,869.4 | 21,929.4 | 10,960.3 |
| page_image_copies | 62.2 | 0.3 | 0.3 | 0.4 | 0.5 | 0.4 |
| page_image_bytes_copied | 254,587.6 | 1,286.2 | 1,182.0 | 1,506.4 | 1,970.9 | 1,768.0 |
| page_image_buffers | 14.8 | 16.0 | 16.0 | 16.0 | 16.0 | 16.0 |
| dirty_page_bytes_copied | 60,787.8 | 627.0 | 572.2 | 733.5 | 958.4 | 859.6 |
| delta_payload_buffers | 32.3 | 32.3 | 32.3 | 32.3 | 32.5 | 32.4 |
| delta_verify_images | 16.1 | 0.2 | 0.1 | 0.2 | 0.2 | 0.2 |
| planner_map_inserts | 1,282.7 | 1,262.0 | 1,250.4 | 1,254.0 | 16.0 | 16.0 |
| planner_key_copies | 48.0 | 48.0 | 48.0 | 48.0 | 32.0 | 32.0 |
| planner_mutation_clones | 16.0 | 16.0 | 16.0 | 16.0 | 0.0 | 0.0 |
| payload_arcs_created | 0.0 | 0.0 | 0.0 | 0.0 | 16.0 | 16.0 |
| harness alloc_calls (not dodb) | 166.1 | 166.1 | 166.1 | 166.1 | 166.1 | 166.1 |

### 64w width1 uniform (per committed transaction; allocator rows exclude the benchmark harness)

| counter | e0 | e1 | e2a | e2b | e3 | final |
|---|---|---|---|---|---|---|
| alloc_calls | 41.8 | 41.9 | 40.0 | 39.0 | 33.5 | 34.6 |
| free_calls | 41.8 | 41.8 | 39.9 | 39.0 | 33.5 | 34.5 |
| alloc_bytes | 10,252.8 | 10,277.6 | 9,535.8 | 10,233.2 | 10,055.4 | 9,371.6 |
| leaf_entry_clones | 30.2 | 30.2 | 15.1 | 15.1 | 15.1 | 15.1 |
| leaf_entry_drops | 31.2 | 31.2 | 16.1 | 16.1 | 16.1 | 16.1 |
| arc_key_clones | 30.2 | 30.2 | 15.1 | 0.0 | 0.0 | 15.1 |
| arc_value_clones | 30.2 | 30.2 | 15.1 | 15.1 | 15.1 | 15.1 |
| arc_key_drops | 31.2 | 31.2 | 16.1 | 0.0 | 0.0 | 16.1 |
| arc_value_drops | 31.2 | 31.2 | 16.1 | 16.1 | 16.1 | 16.1 |
| leaf_page_clones | 2.0 | 2.0 | 1.0 | 1.0 | 1.0 | 1.0 |
| leaf_vec_capacity_bytes | 1,449.3 | 1,449.3 | 724.7 | 1,449.3 | 1,449.3 | 724.8 |
| page_image_copies | 4.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| page_image_bytes_copied | 16,349.6 | 50.8 | 29.4 | 53.3 | 48.9 | 30.2 |
| page_image_buffers | 1.0 | 1.0 | 1.0 | 1.0 | 1.0 | 1.0 |
| dirty_page_bytes_copied | 4,078.8 | 25.4 | 14.7 | 26.6 | 24.4 | 15.1 |
| delta_payload_buffers | 2.0 | 2.0 | 2.0 | 2.0 | 2.0 | 2.0 |
| delta_verify_images | 1.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 |
| planner_map_inserts | 5.0 | 5.0 | 5.0 | 5.0 | 1.0 | 1.0 |
| planner_key_copies | 3.0 | 3.0 | 3.0 | 3.0 | 2.0 | 2.0 |
| planner_mutation_clones | 1.0 | 1.0 | 1.0 | 1.0 | 0.0 | 0.0 |
| payload_arcs_created | 0.0 | 0.0 | 0.0 | 0.0 | 1.0 | 1.0 |
| harness alloc_calls (not dodb) | 16.0 | 16.1 | 16.0 | 16.1 | 16.0 | 16.0 |
