import { S3Client, GetObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { execSync } from "node:child_process";
import { readFileSync, createWriteStream, existsSync, promises as fsp } from "node:fs";
import { rm, mkdir, readdir } from "node:fs/promises";
import { pipeline } from "node:stream/promises";
import path from "node:path";
import { zstdCompressSync } from "node:zlib";
import * as tar from "tar";

export async function handler(event) {
  const s3Client = new S3Client({});

  const cWasmBucketName = process.env.CWASM_BUCKET;
  const wasmtimeVersion = process.env.FN0_WASMTIME_VERSION;
  const fn0WasmtimePath = `/opt/fn0-wasmtime`;

  if (!wasmtimeVersion) {
    throw new Error("FN0_WASMTIME_VERSION is required");
  }

  if (event.Records.length !== 1) {
    throw new Error(`Expected exactly one record, got ${event.Records.length}`);
  }
  const [record] = event.Records;
  const s3Event = JSON.parse(record.body);
  const s3Record = s3Event.Records[0];

  const bucket = s3Record.s3.bucket.name;
  const key = decodeURIComponent(s3Record.s3.object.key.replace(/\+/g, " "));

  console.log(`Processing s3://${bucket}/${key}`);

  if (!key.endsWith(".raw.tar")) {
    console.log(`Ignoring non raw.tar key: ${key}`);
    return;
  }
  if (!key.startsWith("sources/")) {
    console.log(`Ignoring non source bundle key: ${key}`);
    return;
  }

  const inputTarPath = "/tmp/input.tar";
  const extractDir = "/tmp/extract";
  const outputTarPath = "/tmp/output.tar";
  const outputTarZstPath = "/tmp/output.tar.zst";

  console.log("cleanup before start");
  await Promise.all([
    rm(inputTarPath, { force: true }),
    rm(extractDir, { recursive: true, force: true }),
    rm(outputTarPath, { force: true }),
    rm(outputTarZstPath, { force: true }),
  ]);
  await mkdir(extractDir, { recursive: true });

  console.log("download raw.tar");
  const getCommand = new GetObjectCommand({ Bucket: bucket, Key: key });
  const response = await s3Client.send(getCommand);
  if (!response.Body) {
    throw new Error("Empty S3 body");
  }
  await pipeline(response.Body.transformToWebStream(), createWriteStream(inputTarPath));

  console.log("extract tar");
  await tar.x({ file: inputTarPath, cwd: extractDir });

  const backendWasmPath = path.join(extractDir, "backend.wasm");
  if (!existsSync(backendWasmPath)) {
    throw new Error("backend.wasm missing in raw bundle");
  }
  const backendCwasmZstPath = path.join(extractDir, "backend.cwasm.zst");

  console.log("compile wasm -> cwasm.zst");
  execSync(`${fn0WasmtimePath} --zstd "${backendWasmPath}" "${backendCwasmZstPath}"`, {
    stdio: "inherit",
  });

  console.log("remove original backend.wasm");
  await rm(backendWasmPath);

  console.log("list extract dir for repack");
  const entries = await listAll(extractDir);
  const relEntries = entries.map((p) => path.relative(extractDir, p));

  console.log("repack tar");
  await tar.c({ file: outputTarPath, cwd: extractDir }, relEntries);

  console.log("zstd compress tar");
  const tarBytes = readFileSync(outputTarPath);
  const zstBytes = zstdCompressSync(tarBytes);
  await fsp.writeFile(outputTarZstPath, zstBytes);

  const match = key.match(/^sources\/(.+)\/(\d+)\.raw\.tar$/);
  if (!match) {
    throw new Error(`Unexpected source key format: ${key}`);
  }
  const [, subdomain, codeVersion] = match;
  const outKey = `bundles/${wasmtimeVersion}/${subdomain}/${codeVersion}.tar.zst`;

  console.log(`put ${outKey} to ${cWasmBucketName}`);
  const outBytes = readFileSync(outputTarZstPath);
  await s3Client.send(
    new PutObjectCommand({
      Bucket: cWasmBucketName,
      Key: outKey,
      Body: outBytes,
    })
  );
  await Promise.all([
    rm(inputTarPath, { force: true }),
    rm(extractDir, { recursive: true, force: true }),
    rm(outputTarPath, { force: true }),
    rm(outputTarZstPath, { force: true }),
  ]);

  console.log("done");
}

async function listAll(dir) {
  const out = [];
  async function walk(d) {
    const items = await readdir(d, { withFileTypes: true });
    for (const item of items) {
      const full = path.join(d, item.name);
      if (item.isDirectory()) {
        await walk(full);
      } else {
        out.push(full);
      }
    }
  }
  await walk(dir);
  return out;
}
