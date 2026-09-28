import assert from "node:assert/strict";
import test from "node:test";
import {
  TursoGroupProvider,
  type TursoGroupInputs,
} from "../docDb/turso/group.ts";
import {
  TursoDatabaseProvider,
  type TursoDatabaseInputs,
} from "../docDb/turso/database.ts";

const groupInputs: TursoGroupInputs = {
  organizationSlug: "example-org",
  name: "production",
  location: "aws-ap-northeast-1",
};

const databaseInputs: TursoDatabaseInputs = {
  organizationSlug: "example-org",
  name: "production-control",
  group: "production",
};

test("Turso group diff ignores provider serialization changes", async () => {
  const provider = new TursoGroupProvider();

  assert.deepEqual(
    await provider.diff("group-id", groupInputs, { ...groupInputs }),
    { changes: false },
  );
});

test("Turso group diff detects every business input change", async () => {
  const provider = new TursoGroupProvider();

  for (const property of ["organizationSlug", "name", "location"] as const) {
    const changedInputs = { ...groupInputs, [property]: `${groupInputs[property]}-changed` };
    assert.deepEqual(
      await provider.diff("group-id", groupInputs, changedInputs),
      { changes: true },
    );
  }
});

test("Turso group update rejects unsupported changes without an API call", async () => {
  const provider = new TursoGroupProvider();
  const originalFetch = globalThis.fetch;
  let fetchCalls = 0;
  globalThis.fetch = async () => {
    fetchCalls += 1;
    throw new Error("Unexpected network request");
  };

  try {
    await assert.rejects(
      provider.update("group-id", groupInputs, {
        ...groupInputs,
        location: "aws-ap-northeast-2",
      }),
      /Turso group update is unsupported.*location.*No Turso API request was made\./,
    );
    assert.equal(fetchCalls, 0);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("Turso group update accepts unchanged business inputs without an API call", async () => {
  const provider = new TursoGroupProvider();
  const originalFetch = globalThis.fetch;
  let fetchCalls = 0;
  globalThis.fetch = async () => {
    fetchCalls += 1;
    throw new Error("Unexpected network request");
  };

  try {
    const result = await provider.update("group-id", groupInputs, { ...groupInputs });
    assert.deepEqual(result.outs, groupInputs);
    assert.equal(fetchCalls, 0);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("Turso group update accepts provider-only changes when old outputs omit business inputs", async () => {
  const provider = new TursoGroupProvider();
  const originalFetch = globalThis.fetch;
  let fetchCalls = 0;
  globalThis.fetch = async () => {
    fetchCalls += 1;
    throw new Error("Unexpected network request");
  };

  try {
    const result = await provider.update(
      "group-id",
      { __provider: "old" } as TursoGroupInputs,
      { ...groupInputs, __provider: "new" } as TursoGroupInputs,
    );
    assert.deepEqual(result.outs, groupInputs);
    assert.equal(fetchCalls, 0);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("Turso database diff ignores provider serialization changes", async () => {
  const provider = new TursoDatabaseProvider();

  assert.deepEqual(
    await provider.diff("database-id", databaseInputs, { ...databaseInputs }),
    { changes: false },
  );
});

test("Turso database diff detects every business input change", async () => {
  const provider = new TursoDatabaseProvider();

  for (const property of ["organizationSlug", "name", "group"] as const) {
    const changedInputs = {
      ...databaseInputs,
      [property]: `${databaseInputs[property]}-changed`,
    };
    assert.deepEqual(
      await provider.diff("database-id", databaseInputs, changedInputs),
      { changes: true },
    );
  }
});

test("Turso database update rejects unsupported changes without an API call", async () => {
  const provider = new TursoDatabaseProvider();
  const originalFetch = globalThis.fetch;
  let fetchCalls = 0;
  globalThis.fetch = async () => {
    fetchCalls += 1;
    throw new Error("Unexpected network request");
  };

  try {
    await assert.rejects(
      provider.update("database-id", databaseInputs, {
        ...databaseInputs,
        group: "production-next",
      }),
      /Turso database update is unsupported.*group.*No Turso API request was made\./,
    );
    assert.equal(fetchCalls, 0);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("Turso database update accepts unchanged business inputs without an API call", async () => {
  const provider = new TursoDatabaseProvider();
  const originalFetch = globalThis.fetch;
  let fetchCalls = 0;
  globalThis.fetch = async () => {
    fetchCalls += 1;
    throw new Error("Unexpected network request");
  };

  try {
    const result = await provider.update("database-id", databaseInputs, { ...databaseInputs });
    assert.deepEqual(result.outs, databaseInputs);
    assert.equal(fetchCalls, 0);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("Turso database update accepts provider-only changes when old outputs omit business inputs", async () => {
  const provider = new TursoDatabaseProvider();
  const originalFetch = globalThis.fetch;
  let fetchCalls = 0;
  globalThis.fetch = async () => {
    fetchCalls += 1;
    throw new Error("Unexpected network request");
  };

  try {
    const result = await provider.update(
      "database-id",
      { __provider: "old" } as never,
      { ...databaseInputs, __provider: "new" } as TursoDatabaseInputs,
    );
    assert.deepEqual(result.outs, databaseInputs);
    assert.equal(fetchCalls, 0);
  } finally {
    globalThis.fetch = originalFetch;
  }
});
