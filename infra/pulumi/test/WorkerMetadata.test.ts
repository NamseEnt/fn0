import assert from "node:assert/strict";
import test from "node:test";
import * as pulumi from "@pulumi/pulumi";
import {
  buildWorkerInstanceMetadata,
  buildWorkerUserData,
} from "../WorkerMetadata.ts";

test("worker user data and instance metadata are secret outputs", async () => {
  const userData = buildWorkerUserData("#cloud-config\n" as const);
  const metadata = buildWorkerInstanceMetadata(userData, "ssh-ed25519 public");

  assert.equal(await pulumi.isSecret(userData), true);
  assert.equal(await pulumi.isSecret(metadata), true);
});
