import re
import sys


def readable(symbol):
    if not symbol.startswith("_R"):
        return symbol
    names = []
    position = 0
    while position < len(symbol):
        match = re.match(r"(\d+)", symbol[position:])
        if not match:
            position += 1
            continue
        length = int(match.group(1))
        start = position + len(match.group(1))
        if start < len(symbol) and symbol[start] == "_":
            start += 1
        name = symbol[start:start + length]
        if length > 0 and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", name or ""):
            names.append(name)
            position = start + length
        else:
            position += len(match.group(1))
    skip = {"core", "alloc", "std", "llvm", "iter", "traits", "iterator", "adapters", "ops", "function", "vec", "raw_vec",
            "collections", "btree", "map", "set", "clone", "Clone", "drop", "Drop", "ptr", "slice", "tokio", "runtime",
            "task", "harness", "scheduler", "multi_thread", "worker"}
    kept = [name for name in names if name not in skip and not re.fullmatch(r"Cs[0-9A-Za-z_]+", name)]
    return "::".join(kept[-4:]) if kept else symbol[:60]


def main():
    rows = []
    for line in open(sys.argv[1], encoding="utf-8", errors="replace"):
        match = re.match(r"\s+([\d.]+)%\s+([\d.]+)%\s+\[[.k]\]\s+(\S+)", line)
        if match:
            rows.append((float(match.group(1)), float(match.group(2)), readable(match.group(3))))
    print("| children % | self % | function |")
    print("|---|---|---|")
    for children, self_percent, name in rows[: int(sys.argv[2]) if len(sys.argv) > 2 else 45]:
        print(f"| {children:.2f} | {self_percent:.2f} | `{name}` |")


if __name__ == "__main__":
    main()
