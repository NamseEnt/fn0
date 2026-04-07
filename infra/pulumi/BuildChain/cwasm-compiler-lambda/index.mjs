import { S3Client, GetObjectCommand, DeleteObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { execSync } from "node:child_process";
import { readFileSync, createWriteStream } from "node:fs";
import { rm } from "node:fs/promises";
import { pipeline } from "node:stream/promises";
import path from "node:path";

export async function handler(event) {
  const s3Client = new S3Client({});

  const cWasmBucketName = process.env.CWASM_BUCKET;
  const wasmtimePath = `/opt/wasmtime`;
  const zstdPath = `/opt/zstd`;

  if (event.Records.length !== 1) {
    throw new Error(`Expected exactly one record, got ${event.Records.length}`);
  }
  const [record] = event.Records;
  const s3Event = JSON.parse(record.body);
  const s3Record = s3Event.Records[0];

  const bucket = s3Record.s3.bucket.name;
  const key = s3Record.s3.object.key;

  console.log(`Processing s3://${bucket}/${key}`);

  const wasmPath = path.join("/tmp", "input.wasm");
  const cwasmPath = path.join("/tmp", "output.cwasm");
  const cwasmZstdPath = path.join("/tmp", "output.cwasm.zst");

  console.log("clear up before start");
  await Promise.all([
    rm(wasmPath, { force: true }),
    rm(cwasmPath, { force: true }),
    rm(cwasmZstdPath, { force: true }),
  ]);

  console.log("get wasm from s3");
  const getCommand = new GetObjectCommand({
    Bucket: bucket,
    Key: key,
  });
  const response = await s3Client.send(getCommand);
  if (!response.Body) {
    console.log("no body");
    return;
  }
  await pipeline(
    response.Body.transformToWebStream(),
    createWriteStream(wasmPath)
  );

  console.log("compile wasm to cwasm");
  execSync(`${wasmtimePath} compile "${wasmPath}" -o "${cwasmPath}"`, {
    stdio: "inherit",
  });

  console.log("zstd cwasm");
  execSync(`${zstdPath} --ultra -22 "${cwasmPath}"`, {
    stdio: "inherit",
  });

  console.log("put cwasm to s3");
  const cwasmZstdBuffer = readFileSync(cwasmZstdPath);

  const cwasmKey = key.replace(/\.wasm$/, ".cwasm.zst");
  const putCommand = new PutObjectCommand({
    Bucket: cWasmBucketName,
    Key: cwasmKey,
    Body: cwasmZstdBuffer,
  });
  await Promise.all([
    await s3Client.send(putCommand),
    rm(wasmPath),
    rm(cwasmPath),
    rm(cwasmZstdPath),
  ]);

  console.log("delete wasm from s3");
  const deleteCommand = new DeleteObjectCommand({
    Bucket: bucket,
    Key: key,
  });
  await s3Client.send(deleteCommand);
}
