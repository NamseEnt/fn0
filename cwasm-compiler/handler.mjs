import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Readable } from "node:stream";
import {
  S3Client,
  GetObjectCommand,
  PutObjectCommand,
} from "@aws-sdk/client-s3";
import tar from "tar-stream";

const WASMTIME_BIN = "/opt/fn0-wasmtime";

export const handler = async (event) => {
  const inputBucket = event?.input_bucket;
  const inputKey = event?.input_key;
  const outputBucket = event?.output_bucket;
  const outputKey = event?.output_key;

  for (const [name, value] of Object.entries({ inputBucket, inputKey, outputBucket, outputKey })) {
    if (typeof value !== "string" || value.length === 0) {
      throw new Error(`event.${name} must be a non-empty string; got ${JSON.stringify(value)}`);
    }
  }

  const s3 = new S3Client({});

  const obj = await s3.send(
    new GetObjectCommand({ Bucket: inputBucket, Key: inputKey }),
  );
  const rawTarBytes = Buffer.from(await obj.Body.transformToByteArray());

  const entries = await unpackTar(rawTarBytes);

  const backendWasm = entries.get("backend.wasm");
  if (!backendWasm) {
    throw new Error(`raw bundle missing backend.wasm (entries: ${[...entries.keys()].join(", ")})`);
  }

  const workDir = mkdtempSync(join(tmpdir(), "cwasm-"));
  const inputWasmPath = join(workDir, "backend.wasm");
  const outputCwasmZstPath = join(workDir, "backend.cwasm.zst");

  let outputBundleBytes;
  try {
    writeFileSync(inputWasmPath, backendWasm);
    execFileSync(WASMTIME_BIN, ["--zstd", inputWasmPath, outputCwasmZstPath], { stdio: "inherit" });
    const cwasmZst = readFileSync(outputCwasmZstPath);

    const outputEntries = new Map();
    const manifest = entries.get("manifest.json");
    if (!manifest) {
      throw new Error("raw bundle missing manifest.json");
    }
    outputEntries.set("manifest.json", manifest);
    outputEntries.set("backend.cwasm.zst", cwasmZst);

    for (const [name, data] of entries) {
      if (name === "manifest.json" || name === "backend.wasm") continue;
      outputEntries.set(name, data);
    }

    const outputTarBytes = await packTar(outputEntries);
    outputBundleBytes = await zstdCompress(outputTarBytes, workDir);
  } finally {
    rmSync(workDir, { recursive: true, force: true });
  }

  await s3.send(
    new PutObjectCommand({
      Bucket: outputBucket,
      Key: outputKey,
      Body: outputBundleBytes,
    }),
  );

  return { output_bucket: outputBucket, output_key: outputKey, size: outputBundleBytes.length };
};

async function unpackTar(bytes) {
  const result = new Map();
  const extract = tar.extract();

  return new Promise((resolve, reject) => {
    extract.on("entry", (header, stream, next) => {
      const chunks = [];
      stream.on("data", (c) => chunks.push(c));
      stream.on("end", () => {
        if (header.type === "file") {
          result.set(header.name, Buffer.concat(chunks));
        }
        next();
      });
      stream.on("error", reject);
      stream.resume();
    });
    extract.on("finish", () => resolve(result));
    extract.on("error", reject);

    Readable.from(bytes).pipe(extract);
  });
}

async function packTar(entries) {
  const pack = tar.pack();
  for (const [name, data] of entries) {
    pack.entry({ name, size: data.length, mode: 0o644 }, data);
  }
  pack.finalize();

  const chunks = [];
  for await (const chunk of pack) {
    chunks.push(chunk);
  }
  return Buffer.concat(chunks);
}

async function zstdCompress(bytes, workDir) {
  const inPath = join(workDir, "bundle.tar");
  const outPath = join(workDir, "bundle.tar.zst");
  writeFileSync(inPath, bytes);
  execFileSync("zstd", ["-q", "-f", inPath, "-o", outPath], { stdio: "inherit" });
  return readFileSync(outPath);
}
