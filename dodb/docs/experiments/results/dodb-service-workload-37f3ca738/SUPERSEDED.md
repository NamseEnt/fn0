# Superseded service workload attempt

This directory retains the first OCI workload run at implementation SHA `37f3ca73861fb6db3d1d5ec5becac5bb60c290ed`, including all ten one-repetition measurements and two smoke runs. The records completed and their then-current validation passed.

Do not use these records as the adopted comparison. Its final verifier retained all Scan rows in memory. SHA `20b39c5d61d496d03c95225508d07c3b98222856` replaced that path with a 64-row streaming Scan comparison and bounded mismatch/write-history storage. The selected implementation at SHA `cbda6ed41099c569b060884a71bd2f1d98c1df7b` added paired workflow-prefix fingerprints and reran the full OCI set. See [the selected report](../dodb-service-workload-cbda6ed41/REPORT.md).
