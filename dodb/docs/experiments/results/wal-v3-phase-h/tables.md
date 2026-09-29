# Phase H paired benchmark results

| Set | Sync | Writers | Width | Distribution | Reps | G0 tx/s | H1 tx/s | H1/G0 | G0 cycles/tx | H1 cycles/tx | G0/H1 cycles |
|---|---|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| controls | disabled | 64 | 1 | uniform | 3 | 56841 | 55697 | 0.980x | 83565 | 83770 | 0.998x |
| controls | disabled | 64 | 16 | different-leaf-heavy | 3 | 18440 | 18877 | 1.024x | 269489 | 261533 | 1.030x |
| controls | disabled | 64 | 16 | same-leaf-heavy | 3 | 23998 | 23697 | 0.987x | 223604 | 225190 | 0.993x |
| controls | real | 64 | 1 | uniform | 3 | 23373 | 23462 | 1.004x | 111014 | 109885 | 1.010x |
| controls | real | 64 | 16 | different-leaf-heavy | 3 | 9157 | 9493 | 1.037x | 300813 | 293682 | 1.024x |
| controls | real | 64 | 16 | same-leaf-heavy | 3 | 14911 | 14744 | 0.989x | 235491 | 236515 | 0.996x |
| durable | real | 16 | 1 | uniform | 3 | 8437 | 8950 | 1.061x | 200382 | 195826 | 1.023x |
| durable | real | 16 | 16 | uniform | 3 | 3102 | 3174 | 1.023x | 1169435 | 1117680 | 1.046x |
| durable | real | 64 | 1 | uniform | 3 | 23560 | 23495 | 0.997x | 111759 | 111998 | 0.998x |
| durable | real | 64 | 16 | different-leaf-heavy | 3 | 9425 | 9452 | 1.003x | 298171 | 295059 | 1.011x |
| durable | real | 64 | 16 | same-leaf-heavy | 3 | 14929 | 14856 | 0.995x | 234091 | 236879 | 0.988x |
| durable | real | 64 | 16 | uniform | 3 | 4491 | 4434 | 0.987x | 1016661 | 1019650 | 0.997x |
| gate | disabled | 64 | 1 | uniform | 3 | 58573 | 56326 | 0.962x | 81313 | 83482 | 0.974x |
| gate | disabled | 64 | 16 | uniform | 3 | 5707 | 6150 | 1.078x | 984778 | 895007 | 1.100x |
| gate | real | 64 | 1 | uniform | 3 | 23134 | 23066 | 0.997x | 113988 | 113144 | 1.007x |
| gate | real | 64 | 16 | uniform | 3 | 4461 | 4473 | 1.003x | 1028777 | 1019775 | 1.009x |

## Binary provenance

| Variant | Source SHA | Binary SHA256 | Runtime checkout |
|---|---|---|---|
| G0 | `a78472377fa16ed491c18bd2d6770f7dba7e9311` | `33be90a6707544a7725acdb140da8394c4de5affdd5266675720c61ed7499e83` | `/tmp/dodb-phase-h-baseline` |
| H1 | `5998f7c47a15850c40bb40c8c378ac5c0deca061` | `0ac99953510c5f0977b96fd2883547140a284719f45458d5a62869ab8c69badc` | `/home/opc/dodb-wal-v3` |

## H0 call-site attribution, 64w width16 uniform sync-disabled

| Site | Calls/tx | Bytes/tx | Timing/tx |
|---|---:|---:|---:|
| A. Canonical page checksum | 15.982 | 65464 | 29.03 µs worker encode stage |
| B. Full image fingerprint | 15.982 | 65464 | 32.00 µs worker delta stage |
| C. Base page-chain validation | 15.226 | 62366 | 27.93 µs worker base stage |
| D. WAL redo payload checksum | 15.982 | 1709 | 1.316 µs |
| E. Commit digest | 15.982 updates | 1853 | 1.686 µs |
| F. Frame headers and commit payload | 17.982 | 831 | 0.758 µs headers; 0.043 µs commit payload |
| G. Open/recovery/checkpoint | Outside transaction denominator | Full raw-page and frame validation | Per-operation only |

## Profile-derived CRC cycles

| Variant | Transactions | Total sampled cycles/tx | CRC sample share | CRC cycles/tx, estimated |
|---|---:|---:|---:|---:|
| G0 | 58402 | 640068 | 11.17% | 71496 |
| H1 | 59528 | 626483 | 6.38% | 39970 |

CRC estimated cycles reduction: 44.1%; total sampled cycles/tx reduction: 2.1%.
