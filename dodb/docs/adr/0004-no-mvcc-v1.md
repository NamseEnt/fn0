# ADR 0004: No full MVCC in v1

Status: accepted

v1 uses optimistic validation and committed revisions rather than retaining
full historical versions. A deleted key retains only its latest missing
revision in the logical contract so missing-key ABA can be detected.

Transactional range queries are outside v1 scope.

