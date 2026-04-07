import { getSignedUrl } from "@aws-sdk/cloudfront-signer";
import { getSignedUrl as getS3SignedUrl } from "@aws-sdk/s3-request-presigner";
import { S3Client, DeleteObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";

export async function handler(event) {
  const distributionDomain = process.env.DISTRIBUTION_DOMAIN;
  const keyPairId = process.env.CLOUDFRONT_KEY_PAIR_ID;
  const privateKey = process.env.CLOUDFRONT_PRIVATE_KEY;
  const workerUrl = process.env.WORKER_URL;
  const workerSecretKey = process.env.WORKER_SECRET_KEY;
  const targetBuckets = JSON.parse(process.env.TARGET_BUCKETS);

  if (event.Records.length !== 1) {
    throw new Error(`Expected 1 record, got ${event.Records.length}`);
  }

  const [record] = event.Records;
  const s3Event = JSON.parse(record.body);
  const s3Record = s3Event.Records[0];

  const bucket = s3Record.s3.bucket.name;
  const key = s3Record.s3.object.key;

  console.log(`Processing s3://${bucket}/${key}`);

  const url = `https://${distributionDomain}/${key}`;
  const dateLessThan = new Date(Date.now() + 5 * 60 * 1000).toISOString();

  const sourceUrl = getSignedUrl({
    url,
    keyPairId,
    dateLessThan,
    privateKey,
  });

  const targetUrls = await Promise.all(
    targetBuckets.map(async (target) => {
      const s3Client = new S3Client({
        region: target.region,
        endpoint: target.endpoint,
        forcePathStyle: true,
        credentials: {
          accessKeyId: target.accessKeyId,
          secretAccessKey: target.secretAccessKey,
        },
      });

      const command = new PutObjectCommand({
        Bucket: target.bucketName,
        Key: key,
      });

      return await getS3SignedUrl(s3Client, command, {
        expiresIn: 300,
      });
    })
  );

  console.log("Calling Cloudflare Worker");

  const workerResponse = await fetch(workerUrl, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Worker-Secret": workerSecretKey,
    },
    body: JSON.stringify({ sourceUrl, targetUrls }),
  });

  if (!workerResponse.ok) {
    const errorText = await workerResponse.text();
    throw new Error(`Worker failed: ${workerResponse.status} - ${errorText}`);
  }

  const result = await workerResponse.text();
  if (result !== "OK") {
    throw new Error(`Unexpected worker response: ${result}`);
  }

  console.log("Worker succeeded, deleting from S3");

  const s3Client = new S3Client({});
  await s3Client.send(
    new DeleteObjectCommand({
      Bucket: bucket,
      Key: key,
    })
  );

  console.log("Upload complete");
}
