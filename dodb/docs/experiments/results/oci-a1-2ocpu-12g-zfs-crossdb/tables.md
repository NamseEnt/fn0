Seed sets that differ from the dodb Planned run for the same scenario: 0

### Successful logical tx/s (mean of 3 repetitions)

| # | Writers | Width | Distribution | dodb Planned | Turso WAL | Turso MVCC GC-on | RocksDB | Turso MVCC GC-off | RocksDB pipelined |
|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 3,534 | 1,116 | 573 | 10,034 | 142 | 10,011 |
| 1 | 16 | 1 | compact-locality | 3,851 | 1,493 | 372 | 10,913 | — | 9,910 |
| 2 | 16 | 1 | spread-locality | 4,005 | 1,584 | 3,354 | 10,614 | — | 9,875 |
| 3 | 16 | 16 | uniform | 499 | 320 | 476 | 5,723 | 125 | 7,631 |
| 4 | 16 | 16 | compact-locality | 2,149 | 997 | 441 | 6,827 | — | 7,918 |
| 5 | 16 | 16 | spread-locality | 1,871 | 1,001 | 330 | 7,126 | — | 7,986 |
| 6 | 64 | 1 | uniform | 5,289 | 941 | 324 | 26,254 | 57 | 30,302 |
| 7 | 64 | 1 | compact-locality | 8,116 | 2,160 | 269 | 27,378 | — | 29,707 |
| 8 | 64 | 1 | spread-locality | 6,768 | 1,334 | 7,231 | 26,852 | — | 29,888 |
| 9 | 64 | 16 | uniform | 612 | 280 | 289 | 9,573 | 54 | 8,923 |
| 10 | 64 | 16 | compact-locality | 2,747 | 911 | 225 | 11,601 | — | 10,789 |
| 11 | 64 | 16 | spread-locality | 1,980 | 1,045 | 695 | 11,351 | — | 11,205 |
| 12 | 1 | 1 | uniform | 1,083 | 1,197 | 1,291 | 1,512 | — | 1,563 |
| 13 | 1 | 16 | uniform | 280 | 311 | 877 | 1,353 | — | 1,345 |

### Successful mutation ops/s (mean of 3 repetitions)

| # | Writers | Width | Distribution | dodb Planned | Turso WAL | Turso MVCC GC-on | RocksDB | Turso MVCC GC-off | RocksDB pipelined |
|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 3,534 | 1,116 | 573 | 10,034 | 142 | 10,011 |
| 1 | 16 | 1 | compact-locality | 3,851 | 1,493 | 372 | 10,913 | — | 9,910 |
| 2 | 16 | 1 | spread-locality | 4,005 | 1,584 | 3,354 | 10,614 | — | 9,875 |
| 3 | 16 | 16 | uniform | 7,984 | 5,125 | 7,624 | 91,576 | 2,007 | 122,090 |
| 4 | 16 | 16 | compact-locality | 34,377 | 15,951 | 7,063 | 109,224 | — | 126,695 |
| 5 | 16 | 16 | spread-locality | 29,930 | 16,011 | 5,281 | 114,010 | — | 127,780 |
| 6 | 64 | 1 | uniform | 5,289 | 941 | 324 | 26,254 | 57 | 30,302 |
| 7 | 64 | 1 | compact-locality | 8,116 | 2,160 | 269 | 27,378 | — | 29,707 |
| 8 | 64 | 1 | spread-locality | 6,768 | 1,334 | 7,231 | 26,852 | — | 29,888 |
| 9 | 64 | 16 | uniform | 9,795 | 4,486 | 4,623 | 153,162 | 867 | 142,770 |
| 10 | 64 | 16 | compact-locality | 43,955 | 14,569 | 3,601 | 185,614 | — | 172,618 |
| 11 | 64 | 16 | spread-locality | 31,678 | 16,716 | 11,114 | 181,616 | — | 179,274 |
| 12 | 1 | 1 | uniform | 1,083 | 1,197 | 1,291 | 1,512 | — | 1,563 |
| 13 | 1 | 16 | uniform | 4,477 | 4,970 | 14,038 | 21,645 | — | 21,520 |

### Attempted logical tx/s (mean of 3 repetitions)

| # | Writers | Width | Distribution | dodb Planned | Turso WAL | Turso MVCC GC-on | RocksDB | Turso MVCC GC-off | RocksDB pipelined |
|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 3,535 | 1,263 | 573 | 10,034 | 142 | 10,011 |
| 1 | 16 | 1 | compact-locality | 3,852 | 1,640 | 381 | 10,913 | — | 9,910 |
| 2 | 16 | 1 | spread-locality | 4,005 | 1,732 | 3,354 | 10,614 | — | 9,875 |
| 3 | 16 | 16 | uniform | 499 | 463 | 476 | 5,723 | 126 | 7,631 |
| 4 | 16 | 16 | compact-locality | 2,149 | 1,136 | 486 | 6,827 | — | 7,918 |
| 5 | 16 | 16 | spread-locality | 1,871 | 1,137 | 330 | 7,126 | — | 7,986 |
| 6 | 64 | 1 | uniform | 5,289 | 1,535 | 324 | 26,254 | 57 | 30,302 |
| 7 | 64 | 1 | compact-locality | 8,118 | 2,740 | 433 | 27,378 | — | 29,707 |
| 8 | 64 | 1 | spread-locality | 6,769 | 1,927 | 7,231 | 26,852 | — | 29,888 |
| 9 | 64 | 16 | uniform | 612 | 896 | 290 | 9,573 | 54 | 8,923 |
| 10 | 64 | 16 | compact-locality | 2,747 | 1,490 | 493 | 11,601 | — | 10,789 |
| 11 | 64 | 16 | spread-locality | 1,980 | 1,617 | 695 | 11,351 | — | 11,205 |
| 12 | 1 | 1 | uniform | 1,083 | 1,197 | 1,291 | 1,512 | — | 1,563 |
| 13 | 1 | 16 | uniform | 280 | 311 | 877 | 1,353 | — | 1,345 |

### Latency of successful transactions, p50 / p95 / p99 µs (mean of repetition percentiles)

| # | Writers | Width | Distribution | dodb Planned | Turso WAL | Turso MVCC GC-on | RocksDB | Turso MVCC GC-off | RocksDB pipelined |
|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 4,314 / 6,090 / 8,065 | 829 / 1,555 / 48,046 | 23,952 / 59,237 / 88,039 | 1,656 / 2,013 / 2,331 | 51,965 / 400,175 / 661,079 | 1,579 / 1,884 / 2,168 |
| 1 | 16 | 1 | compact-locality | 3,564 / 9,682 / 12,236 | 768 / 1,147 / 30,630 | 7,899 / 163,714 / 227,857 | 1,545 / 1,922 / 2,313 | — | 1,586 / 1,944 / 2,686 |
| 2 | 16 | 1 | spread-locality | 3,908 / 4,664 / 6,760 | 754 / 1,178 / 24,187 | 2,374 / 8,221 / 80,461 | 1,582 / 1,962 / 2,245 | — | 1,602 / 1,920 / 2,252 |
| 3 | 16 | 16 | uniform | 30,684 / 40,533 / 63,042 | 2,741 / 37,414 / 87,973 | 30,477 / 72,275 / 107,066 | 2,632 / 3,908 / 6,714 | 54,071 / 490,292 / 731,617 | 1,876 / 3,080 / 4,732 |
| 4 | 16 | 16 | compact-locality | 6,830 / 11,178 / 12,596 | 965 / 2,512 / 71,837 | 11,875 / 97,405 / 134,945 | 2,268 / 2,870 / 4,129 | — | 1,893 / 2,726 / 4,005 |
| 5 | 16 | 16 | spread-locality | 7,990 / 11,766 / 14,604 | 992 / 5,731 / 72,678 | 10,526 / 219,905 / 877,354 | 2,191 / 2,740 / 3,624 | — | 1,841 / 2,715 / 4,173 |
| 6 | 64 | 1 | uniform | 11,377 / 17,060 / 22,323 | 1,011 / 58,996 / 94,820 | 155,328 / 497,345 / 822,699 | 2,388 / 3,267 / 4,103 | 421,045 / 3,516,015 / 4,798,295 | 1,994 / 3,075 / 3,845 |
| 7 | 64 | 1 | compact-locality | 7,348 / 11,547 / 14,263 | 265 / 27,797 / 85,115 | 9,809 / 364,096 / 554,756 | 2,268 / 3,096 / 3,886 | — | 1,882 / 2,970 / 13,369 |
| 8 | 64 | 1 | spread-locality | 8,897 / 13,078 / 15,495 | 924 / 45,034 / 86,120 | 7,187 / 17,931 / 26,110 | 2,339 / 3,195 / 3,837 | — | 2,004 / 3,066 / 3,756 |
| 9 | 64 | 16 | uniform | 103,147 / 119,823 / 216,045 | 3,557 / 78,361 / 99,846 | 167,801 / 537,277 / 811,649 | 6,360 / 9,734 / 14,653 | 678,595 / 3,746,325 / 5,106,663 | 6,772 / 10,556 / 14,284 |
| 10 | 64 | 16 | compact-locality | 21,931 / 31,209 / 40,224 | 1,114 / 68,647 / 95,433 | 53,287 / 189,457 / 251,714 | 5,252 / 8,092 / 10,869 | — | 5,645 / 8,612 / 11,468 |
| 11 | 64 | 16 | spread-locality | 31,262 / 40,014 / 44,266 | 1,112 / 65,319 / 95,387 | 62,818 / 216,997 / 346,464 | 5,252 / 8,211 / 12,421 | — | 5,411 / 8,389 / 12,041 |
| 12 | 1 | 1 | uniform | 867 / 1,171 / 1,740 | 801 / 1,019 / 1,279 | 749 / 974 / 1,242 | 643 / 823 / 968 | — | 622 / 797 / 943 |
| 13 | 1 | 16 | uniform | 3,272 / 4,483 / 5,598 | 2,719 / 3,616 / 23,873 | 996 / 1,215 / 1,479 | 699 / 925 / 1,135 | — | 701 / 914 / 1,109 |

### CPU (% of one core, mean) and sampled peak RSS (MiB, max of repetitions)

| # | Writers | Width | Distribution | dodb Planned | Turso WAL | Turso MVCC GC-on | RocksDB | Turso MVCC GC-off | RocksDB pipelined |
|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 44% / 788 | 23% / 117 | 184% / 249 | 31% / 53 | 193% / 187 | 28% / 52 |
| 1 | 16 | 1 | compact-locality | 24% / 404 | 20% / 100 | 184% / 192 | 31% / 47 | — | 25% / 46 |
| 2 | 16 | 1 | spread-locality | 35% / 384 | 19% / 111 | 136% / 213 | 31% / 50 | — | 26% / 49 |
| 3 | 16 | 16 | uniform | 76% / 864 | 43% / 131 | 191% / 345 | 62% / 140 | 192% / 268 | 70% / 171 |
| 4 | 16 | 16 | compact-locality | 61% / 344 | 30% / 98 | 169% / 204 | 49% / 92 | — | 53% / 114 |
| 5 | 16 | 16 | spread-locality | 67% / 324 | 31% / 127 | 188% / 228 | 51% / 153 | — | 55% / 179 |
| 6 | 64 | 1 | uniform | 63% / 889 | 43% / 295 | 195% / 473 | 66% / 97 | 195% / 477 | 73% / 103 |
| 7 | 64 | 1 | compact-locality | 45% / 616 | 41% / 294 | 178% / 477 | 62% / 84 | — | 64% / 97 |
| 8 | 64 | 1 | spread-locality | 55% / 560 | 39% / 295 | 144% / 557 | 64% / 96 | — | 69% / 101 |
| 9 | 64 | 16 | uniform | 77% / 952 | 61% / 355 | 191% / 643 | 90% / 258 | 195% / 507 | 91% / 252 |
| 10 | 64 | 16 | compact-locality | 76% / 388 | 49% / 295 | 179% / 486 | 79% / 129 | — | 80% / 117 |
| 11 | 64 | 16 | spread-locality | 83% / 359 | 52% / 416 | 185% / 527 | 77% / 193 | — | 80% / 182 |
| 12 | 1 | 1 | uniform | 21% / 633 | 16% / 45 | 14% / 121 | 7% / 37 | — | 7% / 36 |
| 13 | 1 | 16 | uniform | 43% / 718 | 30% / 47 | 36% / 146 | 16% / 64 | — | 16% / 64 |

### Busy / conflict / retry / abandoned / error counts (sum of 3 measured intervals)

| # | Writers | Width | Distribution | Engine | Attempted tx | Committed tx | Busy | Busy snapshot | Conflicts | Retries | Abandoned | Errors | Verified |
|---:|---:|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | Turso WAL | 19,103 | 16,890 | 39,650 | 0 | 0 | 37,437 | 2,213 | 0 | 3/3 |
| 0 | 16 | 1 | uniform | Turso MVCC GC-on | 9,110 | 9,110 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 0 | 16 | 1 | uniform | RocksDB | 150,550 | 150,550 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 0 | 16 | 1 | uniform | Turso MVCC GC-off | 2,174 | 2,174 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 0 | 16 | 1 | uniform | RocksDB pipelined | 150,204 | 150,204 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 1 | 16 | 1 | compact-locality | Turso WAL | 24,736 | 22,513 | 39,833 | 0 | 0 | 37,610 | 2,223 | 0 | 3/3 |
| 1 | 16 | 1 | compact-locality | Turso MVCC GC-on | 5,863 | 5,716 | 7 | 0 | 4,357 | 4,217 | 147 | 0 | 3/3 |
| 1 | 16 | 1 | compact-locality | RocksDB | 163,735 | 163,735 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 1 | 16 | 1 | compact-locality | RocksDB pipelined | 148,704 | 148,704 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 2 | 16 | 1 | spread-locality | Turso WAL | 26,142 | 23,921 | 39,769 | 0 | 0 | 37,548 | 2,221 | 0 | 3/3 |
| 2 | 16 | 1 | spread-locality | Turso MVCC GC-on | 50,712 | 50,712 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 2 | 16 | 1 | spread-locality | RocksDB | 159,249 | 159,249 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 2 | 16 | 1 | spread-locality | RocksDB pipelined | 148,168 | 148,168 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 3 | 16 | 16 | uniform | Turso WAL | 7,028 | 4,866 | 38,398 | 0 | 0 | 36,236 | 2,162 | 0 | 3/3 |
| 3 | 16 | 16 | uniform | Turso MVCC GC-on | 7,156 | 7,156 | 0 | 0 | 417 | 417 | 0 | 0 | 3/3 |
| 3 | 16 | 16 | uniform | RocksDB | 85,881 | 85,881 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 3 | 16 | 16 | uniform | Turso MVCC GC-off | 1,928 | 1,926 | 0 | 0 | 34 | 32 | 2 | 0 | 3/3 |
| 3 | 16 | 16 | uniform | RocksDB pipelined | 114,492 | 114,492 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 4 | 16 | 16 | compact-locality | Turso WAL | 17,156 | 15,054 | 40,405 | 0 | 0 | 38,303 | 2,102 | 0 | 3/3 |
| 4 | 16 | 16 | compact-locality | Turso MVCC GC-on | 7,344 | 6,674 | 0 | 4 | 31,316 | 30,650 | 670 | 0 | 3/3 |
| 4 | 16 | 16 | compact-locality | RocksDB | 102,437 | 102,437 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 4 | 16 | 16 | compact-locality | RocksDB pipelined | 118,815 | 118,815 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 5 | 16 | 16 | spread-locality | Turso WAL | 17,158 | 15,107 | 40,989 | 0 | 0 | 38,938 | 2,051 | 0 | 3/3 |
| 5 | 16 | 16 | spread-locality | Turso MVCC GC-on | 5,428 | 5,428 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 5 | 16 | 16 | spread-locality | RocksDB | 106,917 | 106,917 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 5 | 16 | 16 | spread-locality | RocksDB pipelined | 119,821 | 119,821 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 6 | 64 | 1 | uniform | Turso WAL | 23,385 | 14,339 | 166,073 | 0 | 0 | 157,027 | 9,046 | 0 | 3/3 |
| 6 | 64 | 1 | uniform | Turso MVCC GC-on | 4,883 | 4,883 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 6 | 64 | 1 | uniform | RocksDB | 393,982 | 393,982 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 6 | 64 | 1 | uniform | Turso MVCC GC-off | 1,006 | 1,006 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 6 | 64 | 1 | uniform | RocksDB pipelined | 454,680 | 454,680 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 7 | 64 | 1 | compact-locality | Turso WAL | 41,666 | 32,854 | 169,367 | 0 | 0 | 160,555 | 8,812 | 0 | 3/3 |
| 7 | 64 | 1 | compact-locality | Turso MVCC GC-on | 7,226 | 4,484 | 0 | 74 | 49,342 | 46,674 | 2,742 | 0 | 3/3 |
| 7 | 64 | 1 | compact-locality | RocksDB | 410,812 | 410,812 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 7 | 64 | 1 | compact-locality | RocksDB pipelined | 445,739 | 445,739 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 8 | 64 | 1 | spread-locality | Turso WAL | 29,327 | 20,306 | 167,377 | 0 | 0 | 158,356 | 9,021 | 0 | 3/3 |
| 8 | 64 | 1 | spread-locality | Turso MVCC GC-on | 115,225 | 115,225 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 8 | 64 | 1 | spread-locality | RocksDB | 403,339 | 403,339 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 8 | 64 | 1 | spread-locality | RocksDB pipelined | 448,460 | 448,460 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 9 | 64 | 16 | uniform | Turso WAL | 13,693 | 4,285 | 160,061 | 0 | 0 | 150,653 | 9,408 | 0 | 3/3 |
| 9 | 64 | 16 | uniform | Turso MVCC GC-on | 4,541 | 4,527 | 0 | 0 | 977 | 963 | 14 | 0 | 3/3 |
| 9 | 64 | 16 | uniform | RocksDB | 143,706 | 143,706 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 9 | 64 | 16 | uniform | Turso MVCC GC-off | 934 | 932 | 0 | 0 | 32 | 30 | 2 | 0 | 3/3 |
| 9 | 64 | 16 | uniform | RocksDB pipelined | 133,943 | 133,943 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 10 | 64 | 16 | compact-locality | Turso WAL | 22,693 | 13,868 | 168,979 | 0 | 0 | 160,154 | 8,825 | 0 | 3/3 |
| 10 | 64 | 16 | compact-locality | Turso MVCC GC-on | 7,571 | 3,453 | 0 | 30 | 86,392 | 82,304 | 4,118 | 0 | 3/3 |
| 10 | 64 | 16 | compact-locality | RocksDB | 174,138 | 174,138 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 10 | 64 | 16 | compact-locality | RocksDB pipelined | 161,970 | 161,970 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 11 | 64 | 16 | spread-locality | Turso WAL | 24,665 | 15,936 | 169,551 | 0 | 0 | 160,822 | 8,729 | 0 | 3/3 |
| 11 | 64 | 16 | spread-locality | Turso MVCC GC-on | 11,461 | 11,461 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 11 | 64 | 16 | spread-locality | RocksDB | 170,488 | 170,488 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 11 | 64 | 16 | spread-locality | RocksDB pipelined | 168,243 | 168,243 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 12 | 1 | 1 | uniform | Turso WAL | 17,961 | 17,961 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 12 | 1 | 1 | uniform | Turso MVCC GC-on | 19,373 | 19,373 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 12 | 1 | 1 | uniform | RocksDB | 22,676 | 22,676 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 12 | 1 | 1 | uniform | RocksDB pipelined | 23,441 | 23,441 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 13 | 1 | 16 | uniform | Turso WAL | 4,689 | 4,689 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 13 | 1 | 16 | uniform | Turso MVCC GC-on | 13,162 | 13,162 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 13 | 1 | 16 | uniform | RocksDB | 20,295 | 20,295 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |
| 13 | 1 | 16 | uniform | RocksDB pipelined | 20,177 | 20,177 | 0 | 0 | 0 | 0 | 0 | 0 | 3/3 |

### Planned / comparator mutation throughput per multiwriter scenario

| # | Writers | Width | Distribution | Planned / Turso WAL | Planned / Turso MVCC GC-on | Planned / RocksDB | Planned / Turso MVCC GC-off | Planned / RocksDB pipelined |
|---:|---:|---:|---|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 3.166x | 6.168x | 0.352x | 24.844x | 0.353x |
| 1 | 16 | 1 | compact-locality | 2.580x | 10.358x | 0.353x | — | 0.389x |
| 2 | 16 | 1 | spread-locality | 2.528x | 1.194x | 0.377x | — | 0.406x |
| 3 | 16 | 16 | uniform | 1.558x | 1.047x | 0.087x | 3.978x | 0.065x |
| 4 | 16 | 16 | compact-locality | 2.155x | 4.867x | 0.315x | — | 0.271x |
| 5 | 16 | 16 | spread-locality | 1.869x | 5.668x | 0.263x | — | 0.234x |
| 6 | 64 | 1 | uniform | 5.619x | 16.330x | 0.201x | 93.384x | 0.175x |
| 7 | 64 | 1 | compact-locality | 3.757x | 30.199x | 0.296x | — | 0.273x |
| 8 | 64 | 1 | spread-locality | 5.073x | 0.936x | 0.252x | — | 0.226x |
| 9 | 64 | 16 | uniform | 2.183x | 2.119x | 0.064x | 11.292x | 0.069x |
| 10 | 64 | 16 | compact-locality | 3.017x | 12.206x | 0.237x | — | 0.255x |
| 11 | 64 | 16 | spread-locality | 1.895x | 2.850x | 0.174x | — | 0.177x |

### Geometric means of Planned / comparator (primary comparators)

| Group | Planned / Turso WAL | Planned / Turso MVCC GC-on | Planned / RocksDB |
|---|---:|---:|---:|
| overall (12) | 2.733x (n=12) | 4.532x (n=12) | 0.221x (n=12) |
| width 1 | 3.610x (n=6) | 5.725x (n=6) | 0.298x (n=6) |
| width 16 | 2.069x (n=6) | 3.587x (n=6) | 0.163x (n=6) |
| writers 16 | 2.250x (n=6) | 3.608x (n=6) | 0.264x (n=6) |
| writers 64 | 3.319x (n=6) | 5.693x (n=6) | 0.185x (n=6) |
| uniform | 2.789x (n=4) | 3.867x (n=4) | 0.141x (n=4) |
| compact locality | 2.818x (n=4) | 11.676x (n=4) | 0.297x (n=4) |
| spread locality | 2.596x (n=4) | 2.061x (n=4) | 0.257x (n=4) |

### Secondary geometric means (not part of the primary headline)

| Group | Planned / RocksDB pipelined | Planned / Turso MVCC GC-off | Turso MVCC GC-on / GC-off | RocksDB default / pipelined |
|---|---:|---:|---:|---:|
| overall (12) | 0.211x (n=12) | 17.967x (n=4) | 4.647x (n=4) | 0.955x (n=12) |
| width 1 | 0.290x (n=6) | 48.167x (n=2) | 4.799x (n=2) | 0.973x (n=6) |
| width 16 | 0.153x (n=6) | 6.702x (n=2) | 4.499x (n=2) | 0.936x (n=6) |
| writers 16 | 0.248x (n=6) | 9.941x (n=2) | 3.911x (n=2) | 0.939x (n=6) |
| writers 64 | 0.179x (n=6) | 32.473x (n=2) | 5.521x (n=2) | 0.971x (n=6) |
| uniform | 0.129x (n=4) | 17.967x (n=4) | 4.647x (n=4) | 0.914x (n=4) |
| compact locality | 0.293x (n=4) | n/a (n=0) | n/a (n=0) | 0.985x (n=4) |
| spread locality | 0.248x (n=4) | n/a (n=0) | n/a (n=0) | 0.967x (n=4) |

### RocksDB write-path counters in the measured interval (mean of repetitions)

| # | Writers | Width | Distribution | Variant | WAL writes | WAL syncs | Writes/sync | Ingest MB | Flushes | Compactions | Compaction read MiB | Compaction write MiB | Stall s / delay+stop count | Max L0 files | Max pending compaction MiB |
|---:|---:|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | RocksDB | 49,667 | 5,769 | 8.70 | 4.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 0 | 16 | 1 | uniform | RocksDB pipelined | 49,667 | 6,206 | 8.07 | 4.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 1 | 16 | 1 | compact-locality | RocksDB | 54,000 | 6,041 | 9.03 | 4.4 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 1 | 16 | 1 | compact-locality | RocksDB pipelined | 49,000 | 6,106 | 8.12 | 4.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 2 | 16 | 1 | spread-locality | RocksDB | 52,667 | 5,910 | 8.98 | 4.3 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 2 | 16 | 1 | spread-locality | RocksDB pipelined | 48,667 | 6,117 | 8.07 | 4.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 3 | 16 | 16 | uniform | RocksDB | 28,000 | 3,437 | 8.33 | 36.3 | 1.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 1 | 0.0 |
| 3 | 16 | 16 | uniform | RocksDB pipelined | 37,667 | 4,776 | 7.99 | 48.4 | 1.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 1 | 0.0 |
| 4 | 16 | 16 | compact-locality | RocksDB | 33,333 | 4,100 | 8.33 | 43.3 | 1.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 1 | 0.0 |
| 4 | 16 | 16 | compact-locality | RocksDB pipelined | 39,333 | 4,911 | 8.07 | 50.2 | 1.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 1 | 0.0 |
| 5 | 16 | 16 | spread-locality | RocksDB | 35,000 | 4,319 | 8.25 | 45.2 | 1.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 1 | 0.0 |
| 5 | 16 | 16 | spread-locality | RocksDB pipelined | 39,333 | 4,940 | 8.09 | 50.6 | 1.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 1 | 0.0 |
| 6 | 64 | 1 | uniform | RocksDB | 131,000 | 3,954 | 33.21 | 10.4 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 6 | 64 | 1 | uniform | RocksDB pipelined | 150,667 | 4,833 | 31.37 | 12.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 7 | 64 | 1 | compact-locality | RocksDB | 136,333 | 4,081 | 33.56 | 10.9 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 7 | 64 | 1 | compact-locality | RocksDB pipelined | 148,333 | 4,728 | 31.46 | 11.8 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 8 | 64 | 1 | spread-locality | RocksDB | 133,667 | 4,033 | 33.34 | 10.7 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 8 | 64 | 1 | spread-locality | RocksDB pipelined | 148,667 | 4,802 | 31.14 | 11.9 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 9 | 64 | 16 | uniform | RocksDB | 47,333 | 1,457 | 32.87 | 60.7 | 1.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 1 | 0.0 |
| 9 | 64 | 16 | uniform | RocksDB pipelined | 44,000 | 2,396 | 18.94 | 56.6 | 1.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 1 | 0.0 |
| 10 | 64 | 16 | compact-locality | RocksDB | 57,333 | 1,787 | 32.49 | 73.5 | 2.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 2 | 0.0 |
| 10 | 64 | 16 | compact-locality | RocksDB pipelined | 53,667 | 2,368 | 22.83 | 68.4 | 2.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 2 | 0.0 |
| 11 | 64 | 16 | spread-locality | RocksDB | 56,333 | 1,762 | 32.26 | 72.0 | 2.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 2 | 0.0 |
| 11 | 64 | 16 | spread-locality | RocksDB pipelined | 55,667 | 2,365 | 23.72 | 71.1 | 2.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 2 | 0.0 |
| 12 | 1 | 1 | uniform | RocksDB | 7,559 | 7,559 | 1.00 | 0.7 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 12 | 1 | 1 | uniform | RocksDB pipelined | 7,814 | 7,814 | 1.00 | 0.7 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 13 | 1 | 16 | uniform | RocksDB | 6,765 | 6,765 | 1.00 | 8.6 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |
| 13 | 1 | 16 | uniform | RocksDB pipelined | 6,726 | 6,726 | 1.00 | 8.6 | 0.0 | 0.0 | 0.0 | 0.0 | 0.000 / 0 | 0 | 0.0 |

### Turso files at the end of the measured interval (mean bytes) and RSS growth

| # | Writers | Width | Distribution | Engine | kv.db | kv.db-wal | kv.db-log | RSS growth after seed (MiB) |
|---:|---:|---:|---|---|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | Turso WAL | 11,952,128 | 36,592,499 | n/a | 70.0 |
| 0 | 16 | 1 | uniform | Turso MVCC GC-on | 10,700,117 | 0 | 1,485,617 | 85.9 |
| 0 | 16 | 1 | uniform | Turso MVCC GC-off | 8,048,640 | 0 | 4,072,826 | 45.7 |
| 1 | 16 | 1 | compact-locality | Turso WAL | 11,972,608 | 6,462,939 | n/a | 66.6 |
| 1 | 16 | 1 | compact-locality | Turso MVCC GC-on | 9,418,069 | 0 | 2,867,171 | 46.2 |
| 2 | 16 | 1 | spread-locality | Turso WAL | 11,972,608 | 32,184,099 | n/a | 68.1 |
| 2 | 16 | 1 | spread-locality | Turso MVCC GC-on | 10,697,387 | 0 | 3,906,527 | 74.7 |
| 3 | 16 | 16 | uniform | Turso WAL | 11,952,128 | 154,527,499 | n/a | 76.2 |
| 3 | 16 | 16 | uniform | Turso MVCC GC-on | 9,374,379 | 0 | 7,355,081 | 202.2 |
| 3 | 16 | 16 | uniform | Turso MVCC GC-off | 12,025,856 | 0 | 989,472 | 137.7 |
| 4 | 16 | 16 | compact-locality | Turso WAL | 11,972,608 | 19,557,672 | n/a | 41.8 |
| 4 | 16 | 16 | compact-locality | Turso MVCC GC-on | 11,976,704 | 0 | 3,788,504 | 51.6 |
| 5 | 16 | 16 | spread-locality | Turso WAL | 11,972,608 | 38,453,365 | n/a | 46.6 |
| 5 | 16 | 16 | spread-locality | Turso MVCC GC-on | 11,976,704 | 0 | 2,784,322 | 90.6 |
| 6 | 64 | 1 | uniform | Turso WAL | 11,952,128 | 31,737,765 | n/a | 161.5 |
| 6 | 64 | 1 | uniform | Turso MVCC GC-on | 8,048,640 | 0 | 4,237,062 | 259.1 |
| 6 | 64 | 1 | uniform | Turso MVCC GC-off | 8,048,640 | 0 | 4,009,597 | 185.2 |
| 7 | 64 | 1 | compact-locality | Turso WAL | 11,972,608 | 11,960,392 | n/a | 210.4 |
| 7 | 64 | 1 | compact-locality | Turso MVCC GC-on | 11,976,704 | 0 | 0 | 210.6 |
| 8 | 64 | 1 | spread-locality | Turso WAL | 11,972,608 | 31,128,005 | n/a | 211.4 |
| 8 | 64 | 1 | spread-locality | Turso MVCC GC-on | 10,697,387 | 0 | 3,600,404 | 253.1 |
| 9 | 64 | 16 | uniform | Turso WAL | 11,952,128 | 137,036,725 | n/a | 298.3 |
| 9 | 64 | 16 | uniform | Turso MVCC GC-on | 12,025,856 | 0 | 1,684,234 | 408.6 |
| 9 | 64 | 16 | uniform | Turso MVCC GC-off | 12,025,856 | 0 | 477,621 | 215.3 |
| 10 | 64 | 16 | compact-locality | Turso WAL | 11,972,608 | 25,653,899 | n/a | 163.9 |
| 10 | 64 | 16 | compact-locality | Turso MVCC GC-on | 11,976,704 | 0 | 1,877,976 | 121.1 |
| 11 | 64 | 16 | spread-locality | Turso WAL | 11,972,608 | 34,555,845 | n/a | 275.3 |
| 11 | 64 | 16 | spread-locality | Turso MVCC GC-on | 11,976,704 | 0 | 0 | 247.1 |
| 12 | 1 | 1 | uniform | Turso WAL | 11,952,128 | 1,163,245 | n/a | 11.4 |
| 12 | 1 | 1 | uniform | Turso MVCC GC-on | 12,025,856 | 0 | 892,753 | 23.6 |
| 13 | 1 | 16 | uniform | Turso WAL | 11,952,128 | 2,906,005 | n/a | 12.2 |
| 13 | 1 | 16 | uniform | Turso MVCC GC-on | 12,025,856 | 0 | 919,364 | 27.7 |

### Post-reopen verification of the core matrix

| Engine | Runs passed | Sampled keys checked | Sampled keys with committed writes |
|---|---:|---:|---:|
| RocksDB | 42/42 | 50,342 | 27,585 |
| RocksDB pipelined | 42/42 | 50,341 | 27,839 |
| Turso MVCC GC-on | 42/42 | 50,350 | 17,234 |
| Turso MVCC GC-off | 12/12 | 15,832 | 4,710 |
| Turso WAL | 42/42 | 50,380 | 14,940 |

## Sustained 120 s results

| Writers | Engine | Successful tx/s | Attempted tx/s | p50 / p95 / p99 µs | CPU % one core | Peak RSS MiB | Max disk MiB | Busy | Conflicts | Abandoned | Errors | Verified |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| 16 | dodb Planned | 2,883 | 2,883 | 5,082 / 8,470 / 10,259 | 52 | 8,626 | 7,775.9 | 0 | 0 | 0 | 0 | reopen not run by phase0-bench; WAL sync counters positive |
| 64 | dodb Planned | 3,944 | 3,944 | 15,130 / 23,035 / 30,477 | 69 | 9,694 | 9,011.1 | 0 | 0 | 0 | 0 | reopen not run by phase0-bench; WAL sync counters positive |
| 16 | RocksDB | 10,367 | 10,367 | 1,611 / 1,984 / 2,331 | 35 | 421 | 163.7 | 0 | 0 | 0 | 0 | True |
| 64 | RocksDB | 25,768 | 25,768 | 2,367 / 3,385 / 4,915 | 66 | 669 | 189.9 | 0 | 0 | 0 | 0 | True |
| 16 | Turso MVCC GC-on | 543 | 543 | 27,707 / 68,005 / 103,130 | 191 | 746 | 124.9 | 0 | 0 | 0 | 0 | True |
| 64 | Turso MVCC GC-on | 275 | 275 | 192,001 / 625,084 / 956,006 | 196 | 886 | 120.6 | 0 | 16 | 1 | 0 | True |
| 16 | Turso WAL | 1,026 | 1,172 | 894 / 1,923 / 54,856 | 27 | 200 | 634.9 | 315,054 | 0 | 17,539 | 0 | True |
| 64 | Turso WAL | 889 | 1,496 | 1,046 / 55,122 / 95,063 | 45 | 433 | 571.4 | 1,312,384 | 0 | 72,919 | 0 | True |

### 10-second windows, 16 writers: successful tx/s (p95 / p99 µs)

| Window | dodb Planned | RocksDB | Turso MVCC GC-on | Turso WAL |
|---:|---:|---:|---:|---:|
| 0-10 s | 3,041 (7,937 / 9,786) | 10,037 (1,985 / 2,453) | 489 (77,489 / 111,964) | 1,036 (1,879 / 55,093) |
| 10-20 s | 2,985 (8,209 / 9,965) | 10,812 (1,933 / 2,308) | 538 (64,651 / 96,002) | 1,041 (1,946 / 54,751) |
| 20-30 s | 2,918 (8,427 / 10,571) | 10,921 (1,929 / 2,260) | 502 (75,006 / 119,641) | 1,033 (1,843 / 44,775) |
| 30-40 s | 2,895 (8,193 / 9,473) | 9,978 (2,085 / 2,444) | 499 (72,271 / 108,016) | 1,032 (2,063 / 55,123) |
| 40-50 s | 2,846 (8,766 / 10,623) | 10,656 (1,961 / 2,324) | 516 (72,005 / 99,888) | 1,051 (1,598 / 44,679) |
| 50-60 s | 2,954 (8,166 / 9,968) | 10,587 (1,973 / 2,274) | 514 (79,983 / 132,175) | 1,007 (1,874 / 44,699) |
| 60-70 s | 2,863 (8,836 / 11,340) | 9,887 (2,000 / 2,340) | 577 (63,983 / 95,996) | 1,025 (1,962 / 54,845) |
| 70-80 s | 2,897 (8,380 / 10,195) | 10,719 (1,945 / 2,309) | 570 (64,061 / 95,984) | 1,026 (2,140 / 64,859) |
| 80-90 s | 2,917 (7,967 / 9,940) | 10,173 (1,987 / 2,308) | 586 (63,861 / 91,993) | 1,031 (1,919 / 64,692) |
| 90-100 s | 2,824 (8,226 / 10,014) | 9,828 (2,017 / 2,318) | 569 (63,671 / 95,987) | 1,015 (1,976 / 54,855) |
| 100-110 s | 2,738 (9,050 / 10,294) | 10,429 (1,995 / 2,321) | 556 (65,724 / 100,398) | 1,019 (1,834 / 45,152) |
| 110-120 s | 2,719 (9,098 / 10,930) | 10,382 (1,980 / 2,275) | 601 (60,790 / 95,485) | 994 (2,035 / 64,562) |

### 10-second windows, 64 writers: successful tx/s (p95 / p99 µs)

| Window | dodb Planned | RocksDB | Turso MVCC GC-on | Turso WAL |
|---:|---:|---:|---:|---:|
| 0-10 s | 4,182 (21,987 / 30,818) | 24,232 (4,469 / 6,428) | 259 (600,007 / 920,009) | 878 (54,711 / 87,217) |
| 10-20 s | 4,530 (20,347 / 28,214) | 25,813 (3,485 / 5,382) | 260 (567,728 / 771,983) | 883 (55,078 / 94,160) |
| 20-30 s | 4,211 (21,016 / 26,033) | 26,121 (3,341 / 4,921) | 264 (663,992 / 1,096,530) | 891 (55,109 / 95,096) |
| 30-40 s | 3,976 (23,138 / 28,806) | 26,453 (3,261 / 3,993) | 288 (576,358 / 868,403) | 904 (54,921 / 94,924) |
| 40-50 s | 3,807 (25,825 / 31,221) | 23,427 (3,779 / 7,527) | 274 (624,038 / 828,014) | 906 (55,231 / 95,038) |
| 50-60 s | 3,862 (23,082 / 26,526) | 26,416 (3,223 / 4,117) | 284 (543,986 / 759,987) | 908 (62,247 / 95,284) |
| 60-70 s | 3,779 (23,995 / 35,587) | 25,928 (3,316 / 5,021) | 271 (725,021 / 948,591) | 852 (55,118 / 94,916) |
| 70-80 s | 3,769 (24,282 / 36,684) | 26,638 (3,164 / 4,069) | 287 (551,998 / 778,100) | 877 (55,286 / 94,906) |
| 80-90 s | 3,876 (23,678 / 28,714) | 26,388 (3,283 / 4,039) | 287 (599,996 / 988,003) | 884 (55,076 / 95,096) |
| 90-100 s | 3,764 (23,174 / 32,174) | 26,043 (3,310 / 4,247) | 260 (688,014 / 1,088,009) | 896 (54,855 / 92,825) |
| 100-110 s | 3,773 (23,675 / 34,375) | 26,619 (3,222 / 3,926) | 274 (747,963 / 1,056,013) | 894 (54,879 / 95,169) |
| 110-120 s | 3,978 (21,367 / 44,725) | 25,136 (3,299 / 4,379) | 288 (587,819 / 1,227,874) | 898 (55,033 / 95,143) |

### RocksDB sustained 16 writers: flush, compaction and stall behaviour

- WAL writes 1,244,000, WAL syncs 139,000, writes per sync 8.95, ingest 100.1 MB, commit groups 139,000.
- cfstats deltas: compaction read 0.0 MiB, flush plus compaction write 73.3 MiB, delays 0, stops 0.
- Flushes 2, compactions 0, compaction read 0.0 MiB, compaction write 0.0 MiB.
- Stall time from dbstats 0.000 s; stall-condition transitions 0 []; samples with write stopped 0, with a delayed write rate 0.
- Max L0 files 3, L0 files at end 3, max pending compaction bytes 0.0 MiB.
- Event timeline (seconds from measured start): 8.2 flush (511186 entries), 68.2 flush (465000 entries)

### RocksDB sustained 64 writers: flush, compaction and stall behaviour

- WAL writes 3,092,000, WAL syncs 93,000, writes per sync 33.19, ingest 245.8 MB, commit groups 93,000.
- cfstats deltas: compaction read 152.7 MiB, flush plus compaction write 249.9 MiB, delays 0, stops 0.
- Flushes 5, compactions 1, compaction read 152.7 MiB, compaction write 75.0 MiB.
- Stall time from dbstats 0.000 s; stall-condition transitions 0 []; samples with write stopped 0, with a delayed write rate 0.
- Max L0 files 4, L0 files at end 3, max pending compaction bytes 152.7 MiB.
- Event timeline (seconds from measured start): 22.8 flush (465252 entries), 46.8 flush (465296 entries), 48.8 compaction L0->L6 (152.7->75.0 MiB, 2004 ms), 71.8 flush (464768 entries), 94.8 flush (464890 entries), 118.8 flush (465036 entries)

