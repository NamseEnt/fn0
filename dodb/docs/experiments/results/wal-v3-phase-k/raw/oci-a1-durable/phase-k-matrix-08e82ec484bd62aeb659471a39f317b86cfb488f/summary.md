| Writers / width / distribution | H1 tx/s | J tx/s | K tx/s | J/H1 | K/J | K/H1 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 16 / 1 / uniform | 9046.8 | 1538.2 | 1656.9 | 0.170x | 1.077x | 0.181x |
| 16 / 16 / uniform | 4107.7 | 1440.4 | 1556.0 | 0.360x | 1.080x | 0.383x |
| 64 / 1 / uniform | 27984.6 | 6475.3 | 6379.4 | 0.231x | 0.990x | 0.229x |
| 64 / 16 / uniform | 5815.3 | 4711.1 | 5568.9 | 0.805x | 1.181x | 0.961x |
| 64 / 16 / compact | 15923.5 | 5558.5 | 5608.9 | 0.349x | 0.994x | 0.346x |
| 64 / 16 / spread | 14208.1 | 4835.9 | 5667.4 | 0.340x | 1.152x | 0.390x |

| Writers / width / distribution | H1 p50/p95/p99 us | J p50/p95/p99 us | K p50/p95/p99 us |
| --- | ---: | ---: | ---: |
| 16 / 1 / uniform | 1652/2515/2858 | 16536/21348/23218 | 5900/20684/24415 |
| 16 / 16 / uniform | 3770/5732/7881 | 18060/20474/21877 | 8540/21176/25682 |
| 64 / 1 / uniform | 2323/2755/3897 | 2464/19059/21294 | 7124/20392/26218 |
| 64 / 16 / uniform | 10521/18934/22353 | 6716/31411/33421 | 10805/21247/26953 |
| 64 / 16 / compact | 4054/6257/7781 | 6427/25235/26615 | 10857/20468/24717 |
| 64 / 16 / spread | 5026/6893/10156 | 6632/28949/31087 | 11050/20166/25072 |

| Primary 64 / 16 / uniform metric (median) | H1 | J | K |
| --- | ---: | ---: | ---: |
| writer blocked ns | 0 | 6201071735 | 2450400586 |
| materialization total ns | 0 | 6157329849 | 8377531685 |
| materialization CPU ns | 0 | 0 | 4219084200 |
| data write ns | 0 | 0 | 895813554 |
| data sync ns | 0 | 0 | 2141408274 |
| checkpoint construction ns | 0 | 0 | 14685692 |
| checkpoint sync ns | 0 | 0 | 600992785 |
| publish pause cumulative ns | 0 | 0 | 69460794 |
| backpressure ns | 0 | 0 | 2736767889 |
| backpressure events | 0 | 0 | 327 |
| materializations | 0 | 270 | 360 |
| segments materialized | 0 | 1080 | 1440 |
| B-link data bytes materialized | 0 | 625991680 | 870006784 |
| WAL bytes reclaimed during materialization | 0 | 128308141 | 0 |
| WAL bytes retained at run end | 0 | 171290 | 186817567 |
| peak overlay segments | 0 | 4 | 8 |
