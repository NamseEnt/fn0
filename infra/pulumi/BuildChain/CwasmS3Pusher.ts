import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";
import * as cloudflare from "@pulumi/cloudflare";
import * as tls from "@pulumi/tls";
import * as random from "@pulumi/random";
import * as fs from "fs/promises";

export interface S3CompatibleBucket {
  endpoint: pulumi.Input<string>;
  region: pulumi.Input<string>;
  bucketName: pulumi.Input<string>;
  // IMPORTANT: Ensure PutObject permission is granted for the access key
  accessKeyId: pulumi.Input<string>;
  secretAccessKey: pulumi.Input<string>;
}

export interface CwasmS3PusherArgs {
  awsS3Region: pulumi.Input<string>;
  zoneId: pulumi.Input<string>;
  targetBuckets: S3CompatibleBucket[];
}

export class CwasmS3Pusher extends pulumi.ComponentResource {
  public readonly cwasmZstBucket: pulumi.Output<string>;

  constructor(
    name: string,
    args: CwasmS3PusherArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:cwasm-s3-pusher", name, args, opts);

    const { awsS3Region, zoneId, targetBuckets } = args;

    const workerSecretKey = new random.RandomPassword(
      "worker-secret-key",
      {
        length: 64,
        special: false,
      },
      { parent: this }
    );

    const zone = cloudflare.Zone.get("zone", zoneId);

    const worker = new cloudflare.WorkersScript(
      "s3-pusher-worker",
      {
        accountId: zone.account.apply((account) => account.id!),
        scriptName: "s3-pusher-worker",
        content: fs.readFile("./cwasm-s3-pusher-workers.js", "utf-8"),
        bindings: [
          {
            name: "WORKER_SECRET_KEY",
            type: "text",
            text: workerSecretKey.result,
          },
        ],
      },
      { parent: this }
    );

    new cloudflare.WorkersRoute(
      "s3-pusher-route",
      {
        zoneId: zone.id,
        pattern: pulumi.interpolate`cwasm-s3-pusher.${zone.name}/*`,
        script: "s3-pusher-worker",
      },
      { parent: this }
    );

    new cloudflare.DnsRecord(
      "s3-pusher-dns",
      {
        zoneId: zone.id,
        name: "cwasm-s3-pusher",
        type: "AAAA",
        content: "100::",
        ttl: 1,
        proxied: true,
      },
      { parent: this }
    );

    new cloudflare.Ruleset(
      "s3-pusher-ruleset",
      {
        zoneId: zone.id,
        name: "s3-pusher-protection",
        kind: "zone",
        phase: "http_request_firewall_custom",
        rules: [
          {
            action: "block",
            expression: pulumi.interpolate`(http.host eq "cwasm-s3-pusher.${zone.name}" and not http.request.headers["x-worker-secret"][0] eq "${workerSecretKey.result}")`,
          },
        ],
      },
      { parent: this }
    );

    const cwasmZstBucket = new aws.s3.Bucket(
      "cwasm-zst-bucket",
      {
        region: awsS3Region,
      },
      { parent: this }
    );
    this.cwasmZstBucket = cwasmZstBucket.bucket;

    new aws.s3.BucketLifecycleConfiguration(
      "cwasm-zst-lifecycle",
      {
        region: awsS3Region,
        bucket: cwasmZstBucket.bucket,
        rules: [
          {
            id: "expire-1day",
            status: "Enabled",
            expiration: {
              days: 1,
            },
          },
        ],
      },
      { parent: this }
    );

    const queue = new aws.sqs.Queue(
      "cwasm-upload-queue",
      {
        region: awsS3Region,
        fifoQueue: true,
        visibilityTimeoutSeconds: 90,
      },
      { parent: this }
    );

    // Create SQS policy
    const queuePolicyDoc = aws.iam.getPolicyDocumentOutput(
      {
        statements: [
          {
            effect: "Allow",
            principals: [
              {
                type: "Service",
                identifiers: ["s3.amazonaws.com"],
              },
            ],
            actions: ["sqs:SendMessage"],
            resources: [queue.arn],
            conditions: [
              {
                test: "ArnEquals",
                variable: "aws:SourceArn",
                values: [cwasmZstBucket.arn],
              },
            ],
          },
        ],
      },
      { parent: this }
    );

    const queuePolicy = new aws.sqs.QueuePolicy(
      "cwasm-queue-policy",
      {
        region: awsS3Region,
        queueUrl: queue.id,
        policy: queuePolicyDoc.json,
      },
      { parent: this }
    );

    // Create S3 bucket notification
    new aws.s3.BucketNotification(
      "cwasm-notification",
      {
        region: awsS3Region,
        bucket: cwasmZstBucket.bucket,
        queues: [
          {
            queueArn: queue.arn,
            events: ["s3:ObjectCreated:*"],
          },
        ],
      },
      { dependsOn: [queuePolicy] }
    );

    // Create CloudFront Origin Access Control
    const originAccessControl = new aws.cloudfront.OriginAccessControl(
      "cwasm-oac",
      {
        name: "cwasm-zst-oac",
        originAccessControlOriginType: "s3",
        signingBehavior: "always",
        signingProtocol: "sigv4",
      }
    );

    // Create CloudFront key pair
    const privateKey = new tls.PrivateKey(
      "cloudfront-key",
      {
        algorithm: "RSA",
        rsaBits: 2048,
      },
      { parent: this }
    );

    const publicKey = new aws.cloudfront.PublicKey(
      "cloudfront-public-key",
      {
        encodedKey: privateKey.publicKeyPem,
        name: "cwasm-key-pair",
        comment: "Key for signing cwasm.zst URLs",
      },
      { parent: this }
    );

    const keyGroup = new aws.cloudfront.KeyGroup(
      "cloudfront-key-group",
      {
        items: [publicKey.id],
        name: "cwasm-key-group",
        comment: "Trusted key group for cwasm.zst",
      },
      { parent: this }
    );

    // Create CloudFront distribution
    const distribution = new aws.cloudfront.Distribution(
      "cwasm-distribution",
      {
        origins: [
          {
            domainName: cwasmZstBucket.bucketRegionalDomainName,
            originAccessControlId: originAccessControl.id,
            originId: "cwasmS3Origin",
          },
        ],
        enabled: true,
        defaultCacheBehavior: {
          targetOriginId: "cwasmS3Origin",
          viewerProtocolPolicy: "https-only",
          allowedMethods: ["GET", "HEAD"],
          cachedMethods: ["GET", "HEAD"],
          trustedKeyGroups: [keyGroup.id],
          forwardedValues: {
            queryString: false,
            cookies: {
              forward: "none",
            },
          },
          minTtl: 0,
          defaultTtl: 0,
          maxTtl: 0,
        },
        restrictions: {
          geoRestriction: {
            restrictionType: "none",
          },
        },
        viewerCertificate: {
          cloudfrontDefaultCertificate: true,
        },
      },
      { parent: this }
    );

    // Update S3 bucket policy
    const bucketPolicyDoc = pulumi
      .all([cwasmZstBucket.arn, distribution.arn])
      .apply(([bucketArn, distributionArn]) =>
        JSON.stringify({
          Version: "2012-10-17",
          Statement: [
            {
              Sid: "AllowCloudFrontServicePrincipal",
              Effect: "Allow",
              Principal: {
                Service: "cloudfront.amazonaws.com",
              },
              Action: "s3:GetObject",
              Resource: `${bucketArn}/*`,
              Condition: {
                StringEquals: {
                  "AWS:SourceArn": distributionArn,
                },
              },
            },
          ],
        } satisfies aws.iam.PolicyDocument)
      );

    new aws.s3.BucketPolicy(
      "cwasm-bucket-policy",
      {
        bucket: cwasmZstBucket.id,
        policy: bucketPolicyDoc,
      },
      { parent: this }
    );

    // Create Lambda IAM role
    const cwasmZstBucketArn = pulumi.interpolate`arn:aws:s3:::${cwasmZstBucket.bucket}`;

    const lambdaRole = new aws.iam.Role(
      "s3-pusher-lambda-role",
      {
        assumeRolePolicy: {
          Version: "2012-10-17",
          Statement: [
            {
              Effect: "Allow",
              Principal: {
                Service: "lambda.amazonaws.com",
              },
              Action: "sts:AssumeRole",
            },
          ],
        },
        inlinePolicies: [
          {
            name: "s3-cloudfront-access",
            policy: cwasmZstBucketArn.apply((bucketArn) =>
              JSON.stringify({
                Version: "2012-10-17",
                Statement: [
                  {
                    Effect: "Allow",
                    Action: ["s3:GetObject", "s3:DeleteObject"],
                    Resource: `${bucketArn}/*`,
                  },
                  {
                    Effect: "Allow",
                    Action: ["s3:ListBucket"],
                    Resource: bucketArn,
                  },
                ],
              } satisfies aws.iam.PolicyDocument)
            ),
          },
        ],
        managedPolicyArns: [
          aws.iam.ManagedPolicy.AWSLambdaBasicExecutionRole,
          aws.iam.ManagedPolicy.AWSLambdaSQSQueueExecutionRole,
        ],
      },
      { parent: this }
    );

    const lambda = new aws.lambda.CallbackFunction<aws.sqs.QueueEvent, void>(
      "s3-pusher-lambda",
      {
        region: awsS3Region,
        timeout: 60,
        memorySize: 128,
        environment: {
          variables: {
            DISTRIBUTION_DOMAIN: distribution.domainName,
            CLOUDFRONT_KEY_PAIR_ID: publicKey.id,
            CLOUDFRONT_PRIVATE_KEY: privateKey.privateKeyPem,
            WORKER_URL: pulumi.interpolate`https://cwasm-s3-pusher.${zone.name}/upload`,
            WORKER_SECRET_KEY: workerSecretKey.result,
            TARGET_BUCKETS: pulumi.jsonStringify(targetBuckets),
          },
        },
        callback: async (event) => {
          const { getSignedUrl } = await import("@aws-sdk/cloudfront-signer");
          const { getSignedUrl: getS3SignedUrl } = await import(
            "@aws-sdk/s3-request-presigner"
          );
          const { S3Client, DeleteObjectCommand, PutObjectCommand } =
            await import("@aws-sdk/client-s3");

          const distributionDomain = process.env.DISTRIBUTION_DOMAIN!;
          const keyPairId = process.env.CLOUDFRONT_KEY_PAIR_ID!;
          const privateKey = process.env.CLOUDFRONT_PRIVATE_KEY!;
          const workerUrl = process.env.WORKER_URL!;
          const workerSecretKey = process.env.WORKER_SECRET_KEY!;
          const targetBuckets = JSON.parse(process.env.TARGET_BUCKETS!);

          if (event.Records.length !== 1) {
            throw new Error(`Expected 1 record, got ${event.Records.length}`);
          }

          const [record] = event.Records;
          const bucketRecord = JSON.parse(record.body) as aws.s3.BucketRecord;
          const bucket = bucketRecord.s3.bucket.name;
          const key = bucketRecord.s3.object.key;

          console.log(`Processing s3://${bucket}/${key}`);

          const url = `https://${distributionDomain}/${key}`;
          const dateLessThan = new Date(
            Date.now() + 5 * 60 * 1000
          ).toISOString();

          const sourceUrl = getSignedUrl({
            url,
            keyPairId,
            dateLessThan,
            privateKey,
          });

          const targetUrls = await Promise.all(
            targetBuckets.map(async (target: any) => {
              const s3Client = new S3Client({
                region: target.region,
                endpoint: target.endpoint,
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
            throw new Error(
              `Worker failed: ${workerResponse.status} - ${errorText}`
            );
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
        },
        role: lambdaRole.arn,
      }
    );

    new aws.lambda.EventSourceMapping(
      "s3-pusher-trigger",
      {
        region: awsS3Region,
        eventSourceArn: queue.arn,
        functionName: lambda.name,
        batchSize: 1,
      },
      { parent: this }
    );
  }
}
