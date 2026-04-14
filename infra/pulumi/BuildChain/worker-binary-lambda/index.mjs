import { S3Client, GetObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { spawnSync } from "node:child_process";
import { promises as fsp, createWriteStream, readFileSync, existsSync } from "node:fs";
import { pipeline } from "node:stream/promises";

const s3 = new S3Client({});

export async function handler(event) {
  const { bucket, sourceKey, cacheKey, outputKey } = event;
  if (!bucket || !sourceKey || !outputKey) {
    throw new Error("bucket, sourceKey, outputKey required");
  }

  const srcDir = "/tmp/src";
  const cargoHome = "/tmp/.cargo";
  const sourceTar = "/tmp/source.tar.zst";
  const cacheTar = "/tmp/cache.tar.zst";
  const outputTar = "/tmp/cache-out.tar.zst";

  await cleanup([srcDir, cargoHome, sourceTar, cacheTar, outputTar]);
  await fsp.mkdir(srcDir, { recursive: true });
  await fsp.mkdir(cargoHome, { recursive: true });

  console.log(`[step] download source s3://${bucket}/${sourceKey}`);
  await downloadS3(bucket, sourceKey, sourceTar);
  runOrThrow("tar", ["--zstd", "-xf", sourceTar, "-C", srcDir]);

  if (cacheKey) {
    console.log(`[step] restore cache s3://${bucket}/${cacheKey}`);
    try {
      await downloadS3(bucket, cacheKey, cacheTar);
      runOrThrow("tar", ["--zstd", "-xf", cacheTar, "-C", "/tmp"]);
    } catch (err) {
      if (err?.name === "NoSuchKey") {
        console.log("[step] no cache yet, skipping restore");
      } else {
        console.log(`[step] cache restore failed (continuing): ${err.message}`);
      }
    }
  }

  console.log("[step] cargo build --release");
  const build = spawnSync(
    "cargo",
    ["build", "--release", "--manifest-path", `${srcDir}/fn0-worker/Cargo.toml`],
    {
      stdio: "inherit",
      env: {
        ...process.env,
        CARGO_HOME: cargoHome,
        CARGO_TARGET_DIR: `${srcDir}/target`,
      },
    },
  );
  if (build.status !== 0) {
    throw new Error(`cargo build exited with status ${build.status}`);
  }

  const binaryPath = `${srcDir}/target/release/fn0-worker`;
  if (!existsSync(binaryPath)) {
    throw new Error(`binary missing after build: ${binaryPath}`);
  }

  console.log(`[step] upload binary s3://${bucket}/${outputKey}`);
  await uploadS3(bucket, outputKey, binaryPath);

  if (cacheKey) {
    console.log("[step] pack cache");
    runOrThrow("tar", [
      "--zstd",
      "-cf",
      outputTar,
      "-C",
      "/tmp",
      "src/target",
      ".cargo",
    ]);
    console.log(`[step] upload cache s3://${bucket}/${cacheKey}`);
    await uploadS3(bucket, cacheKey, outputTar);
  }

  await cleanup([srcDir, cargoHome, sourceTar, cacheTar, outputTar]);

  return { binaryKey: outputKey };
}

function runOrThrow(cmd, args) {
  const r = spawnSync(cmd, args, { stdio: "inherit" });
  if (r.status !== 0) {
    throw new Error(`${cmd} ${args.join(" ")} exited with status ${r.status}`);
  }
}

async function downloadS3(bucket, key, dst) {
  const resp = await s3.send(new GetObjectCommand({ Bucket: bucket, Key: key }));
  await pipeline(resp.Body, createWriteStream(dst));
}

async function uploadS3(bucket, key, file) {
  const body = readFileSync(file);
  await s3.send(new PutObjectCommand({ Bucket: bucket, Key: key, Body: body }));
}

async function cleanup(paths) {
  for (const p of paths) {
    await fsp.rm(p, { recursive: true, force: true });
  }
}
