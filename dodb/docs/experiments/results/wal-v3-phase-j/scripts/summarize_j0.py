import json
import pathlib
import sys


source_path = pathlib.Path(sys.argv[1])
artifact_root = source_path.resolve().parents[2]
rows = [json.loads(line) for line in source_path.read_text(encoding="utf-8").splitlines()]
expected_commit = "ffe774c58da1157f6e735ee75262efd978022757"
if not rows or {row["git_commit"] for row in rows} != {expected_commit}:
    raise SystemExit("unexpected or mixed J0 source commit")

lines = [
    "# J0 Read Tables",
    "",
    f"Source commit: `{expected_commit}`. Latencies are nanoseconds per operation.",
    "",
    "## GET",
    "",
    "| Segments | Case | H1 ns | J0 ns | Ratio | H1 alloc/op | J0 alloc/op |",
    "| ---: | --- | ---: | ---: | ---: | ---: | ---: |",
]
for row in rows:
    if row["record_type"] == "get":
        lines.append(
            f"| {row['segments']} | {row['case']} | {row['h1_ns']} | {row['j0_ns']} | {row['ratio']:.3f}x | {row['h1_allocations_per_op']:.0f} | {row['j0_allocations_per_op']:.0f} |"
        )

lines.extend(
    [
        "",
        "## Query and Scan",
        "",
        "| Segments | Operation | Limit | H1 ns | J0 ns | Ratio | H1 alloc/op | J0 alloc/op |",
        "| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
)
for row in rows:
    if row["record_type"] == "range":
        lines.append(
            f"| {row['segments']} | {row['operation']} | {row['limit']} | {row['h1_ns']} | {row['j0_ns']} | {row['ratio']:.3f}x | {row['h1_allocations_per_op']:.0f} | {row['j0_allocations_per_op']:.0f} |"
        )

artifact_root.joinpath("tables.md").write_text(
    "\n".join(lines) + "\n", encoding="utf-8"
)
