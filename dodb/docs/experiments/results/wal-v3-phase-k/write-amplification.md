# Phase K Write Amplification

All rows are three-run medians from the OCI A1/ZFS real-sync H1/J/K matrix. Each transaction writes a 16-byte key and a 64-byte value. Logical user bytes are `mutation_ops * (key_size + value_size)`. WAL bytes use `planner_wal_bytes`, which reports appended bytes for all three selectors; the older `wal_bytes_delta` field does not count the J v4 WAL path correctly.

| Workload | Mode | Logical user bytes | WAL bytes | WAL amp | Materialized B-link bytes | Physical amp | Known subtotal bytes | Known subtotal amp |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 16 / 1 / uniform | h1 | 7239600 | 20416595 | 2.820x | n/a | n/a | n/a | n/a |
| 16 / 1 / uniform | phase-j | 1231920 | 3712313 | 3.013x | 1003126784 | 814.279x | 1006839097 | 817.293x |
| 16 / 1 / uniform | phase-k | 1326560 | 3997499 | 3.013x | 901898240 | 679.877x | 905895739 | 682.891x |
| 16 / 16 / uniform | h1 | 52601600 | 110601807 | 2.103x | n/a | n/a | n/a | n/a |
| 16 / 16 / uniform | phase-j | 18466560 | 39200402 | 2.123x | 932675584 | 50.506x | 971875986 | 52.629x |
| 16 / 16 / uniform | phase-k | 19943680 | 42335997 | 2.123x | 853647360 | 42.803x | 895983357 | 44.926x |
| 64 / 1 / uniform | h1 | 22403680 | 63098833 | 2.816x | n/a | n/a | n/a | n/a |
| 64 / 1 / uniform | phase-j | 5185360 | 15625685 | 3.013x | 995196928 | 191.924x | 1010822613 | 194.938x |
| 64 / 1 / uniform | phase-k | 5112080 | 15404837 | 3.013x | 874086400 | 170.984x | 889491237 | 173.998x |
| 64 / 16 / uniform | h1 | 74504960 | 155432027 | 2.086x | n/a | n/a | n/a | n/a |
| 64 / 16 / uniform | phase-j | 60444160 | 128308132 | 2.123x | 625991680 | 10.357x | 754299812 | 12.479x |
| 64 / 16 / uniform | phase-k | 71479040 | 151732731 | 2.123x | 870006784 | 12.171x | 1021739515 | 14.294x |
| 64 / 16 / compact | h1 | 204019200 | 87069936 | 0.427x | n/a | n/a | n/a | n/a |
| 64 / 16 / compact | phase-j | 71207680 | 152887924 | 2.147x | 734625792 | 10.317x | 887513716 | 12.464x |
| 64 / 16 / compact | phase-k | 71950080 | 154481907 | 2.147x | 826183680 | 11.483x | 980665587 | 13.630x |
| 64 / 16 / spread | h1 | 181966080 | 96455984 | 0.530x | n/a | n/a | n/a | n/a |
| 64 / 16 / spread | phase-j | 61984000 | 130850408 | 2.111x | 656056320 | 10.584x | 786906728 | 12.695x |
| 64 / 16 / spread | phase-k | 72627200 | 153318302 | 2.111x | 855543808 | 11.780x | 1008862110 | 13.891x |

`materialized_data_bytes_delta` counts replacement B-link data bytes for J and K. H1 does not expose a separate physical data-byte counter, so its physical and combined amplification are unavailable rather than zero. Checkpoint bytes are also not separately reported by the current benchmark schema. The known subtotal is WAL plus replacement B-link data and excludes checkpoint bytes; it is not a complete total-engine write count. K's checkpoint sync time is measured separately in the pause metrics.

The primary case K wrote 151.7 MB of logical WAL and 870.0 MB of materialized B-link data for 71.5 MB of logical user key/value bytes: WAL amplification 2.123x, physical materialization amplification 12.171x, and known subtotal amplification 14.294x before checkpoint bytes. J's corresponding known subtotal was 12.479x. K retained a median 186.8 MB of WAL after the primary measured interval and reclaimed zero WAL bytes while newer overlays continued to commit.
