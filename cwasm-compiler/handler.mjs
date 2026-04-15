import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  S3Client,
  GetObjectCommand,
  PutObjectCommand,
} from "@aws-sdk/client-s3";

const BUCKET = process.env.BUCKET;
const WASMTIME_BIN = "/opt/fn0-wasmtime";
const s3 = new S3Client({});

export const handler = async (event) => {
  const key = event?.key;
  if (typeof key !== "string" || !key.startsWith("wasm/") || !key.endsWith(".wasm")) {
    throw new Error(`event.key must be "wasm/<name>.wasm", got: ${JSON.stringify(key)}`);
  }

  const workDir = mkdtempSync(join(tmpdir(), "cwasm-"));
  const inputPath = join(workDir, "input.wasm");
  const outputPath = join(workDir, "output.cwasm.zst");

  try {
    const obj = await s3.send(new GetObjectCommand({ Bucket: BUCKET, Key: key }));
    const bytes = await obj.Body.transformToByteArray();
    writeFileSync(inputPath, Buffer.from(bytes));

    execFileSync(WASMTIME_BIN, ["--zstd", inputPath, outputPath], { stdio: "inherit" });

    const outputKey = key
      .replace(/^wasm\//, "cwasm/")
      .replace(/\.wasm$/, ".cwasm.zst");
    const outputBody = readFileSync(outputPath);
    await s3.send(
      new PutObjectCommand({
        Bucket: BUCKET,
        Key: outputKey,
        Body: outputBody,
      })
    );

    return { outputKey, size: outputBody.length };
  } finally {
    rmSync(workDir, { recursive: true, force: true });
  }
};
