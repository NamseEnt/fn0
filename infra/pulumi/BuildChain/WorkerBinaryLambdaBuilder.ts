import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";
import * as command from "@pulumi/command";
import * as dockerBuild from "@pulumi/docker-build";
import * as path from "node:path";
import * as fs from "node:fs";
import * as crypto from "node:crypto";

export interface WorkerBinaryLambdaBuilderArgs {
  region: pulumi.Input<string>;
}

export class WorkerBinaryLambdaBuilder extends pulumi.ComponentResource {
  public readonly stageDir: pulumi.Output<string>;
  public readonly sourceHash: pulumi.Output<string>;

  constructor(
    name: string,
    args: WorkerBinaryLambdaBuilderArgs,
    opts: pulumi.ComponentResourceOptions,
  ) {
    super("pkg:index:worker-binary-lambda-builder", name, args, opts);

    const { region } = args;

    const artifactBucket = new aws.s3.Bucket(
      "worker-build-bucket",
      { region },
      { parent: this },
    );

    new aws.s3.BucketLifecycleConfiguration(
      "worker-build-bucket-lifecycle",
      {
        region,
        bucket: artifactBucket.bucket,
        rules: [
          {
            id: "expire-sources-and-bins",
            status: "Enabled",
            filter: { prefix: "sources/" },
            expiration: { days: 7 },
          },
          {
            id: "expire-old-binaries",
            status: "Enabled",
            filter: { prefix: "bin/" },
            expiration: { days: 7 },
          },
        ],
      },
      { parent: this },
    );

    const ecrRepo = new aws.ecr.Repository(
      "worker-builder-repo",
      {
        region,
        imageTagMutability: "MUTABLE",
        forceDelete: true,
      },
      { parent: this },
    );

    const authToken = aws.ecr.getAuthorizationTokenOutput({ region });

    const lambdaDir = path.join(__dirname, "worker-binary-lambda");
    const lambdaContextHash = hashDir(lambdaDir);

    const lambdaImage = new dockerBuild.Image(
      "worker-builder-image",
      {
        tags: [pulumi.interpolate`${ecrRepo.repositoryUrl}:${lambdaContextHash}`],
        context: { location: lambdaDir },
        dockerfile: { location: path.join(lambdaDir, "Dockerfile") },
        platforms: [dockerBuild.Platform.Linux_arm64],
        push: true,
        registries: [
          {
            address: ecrRepo.repositoryUrl,
            username: authToken.userName,
            password: authToken.password,
          },
        ],
      },
      { parent: this },
    );

    const bucketArn = pulumi.interpolate`arn:aws:s3:::${artifactBucket.bucket}`;

    const lambdaRole = new aws.iam.Role(
      "worker-builder-lambda-role",
      {
        assumeRolePolicy: {
          Version: "2012-10-17",
          Statement: [
            {
              Effect: "Allow",
              Principal: { Service: "lambda.amazonaws.com" },
              Action: "sts:AssumeRole",
            },
          ],
        },
        inlinePolicies: [
          {
            name: "s3-access-policy",
            policy: bucketArn.apply((arn) =>
              JSON.stringify({
                Version: "2012-10-17",
                Statement: [
                  {
                    Effect: "Allow",
                    Action: [
                      "s3:GetObject",
                      "s3:PutObject",
                      "s3:DeleteObject",
                    ],
                    Resource: `${arn}/*`,
                  },
                  {
                    Effect: "Allow",
                    Action: ["s3:ListBucket"],
                    Resource: arn,
                  },
                ],
              } satisfies aws.iam.PolicyDocument),
            ),
          },
        ],
        managedPolicyArns: [aws.iam.ManagedPolicy.AWSLambdaBasicExecutionRole],
      },
      { parent: this },
    );

    const lambda = new aws.lambda.Function(
      "worker-builder-lambda",
      {
        region,
        packageType: "Image",
        imageUri: pulumi.interpolate`${ecrRepo.repositoryUrl}:${lambdaContextHash}`,
        architectures: ["arm64"],
        memorySize: 10240,
        timeout: 900,
        ephemeralStorage: { size: 10240 },
        role: lambdaRole.arn,
      },
      { parent: this, dependsOn: [lambdaImage] },
    );

    const repoRoot = path.resolve(__dirname, "..", "..", "..");
    const sources = ["host-hq-protocol", "fn0", "fn0-wasmtime", "ski", "fn0-worker"];
    const sourceHash = hashSourceDirs(repoRoot, sources);
    const localSourceTar = `/tmp/fn0-worker-src-${sourceHash}.tar.zst`;

    const sourceTarCmd = new command.local.Command(
      "worker-builder-source-tar",
      {
        create: pulumi.interpolate`rm -f ${localSourceTar} && tar --zstd -C ${repoRoot} -cf ${localSourceTar} ${sources.join(" ")}`,
        triggers: [sourceHash],
      },
      { parent: this },
    );

    const sourceObject = new aws.s3.BucketObject(
      "worker-builder-source",
      {
        region,
        bucket: artifactBucket.bucket,
        key: `sources/${sourceHash}.tar.zst`,
        source: sourceTarCmd.stdout.apply(() => new pulumi.asset.FileAsset(localSourceTar)),
      },
      { parent: this, dependsOn: [sourceTarCmd] },
    );

    const outputKey = `bin/${sourceHash}/fn0-worker`;

    const invocation = new aws.lambda.Invocation(
      "worker-builder-invoke",
      {
        region,
        functionName: lambda.name,
        input: pulumi.jsonStringify({
          bucket: artifactBucket.bucket,
          sourceKey: sourceObject.key,
          cacheKey: "cache/target.tar.zst",
          outputKey,
        }),
        triggers: { sourceHash },
      },
      { parent: this, dependsOn: [sourceObject] },
    );

    const workerImageDir = path.join(__dirname, "worker-binary-image");
    const workerImageHash = hashDir(workerImageDir);
    const sourceDockerfile = path.join(workerImageDir, "Dockerfile");
    const stageDir = `/tmp/fn0-worker-stage-${sourceHash}-${workerImageHash}`;
    const localBinary = `${stageDir}/fn0-worker`;
    const stagedDockerfile = `${stageDir}/Dockerfile`;

    const fetchScript = pulumi.interpolate`set -e
mkdir -p ${stageDir}
if [ ! -s ${localBinary} ]; then
  aws s3 cp --region ${region} s3://${artifactBucket.bucket}/${outputKey} ${localBinary}
fi
chmod +x ${localBinary}
cp ${sourceDockerfile} ${stagedDockerfile}`;

    const fetchBinary = new command.local.Command(
      "worker-builder-fetch-binary",
      {
        create: fetchScript,
        update: fetchScript,
        triggers: [invocation.result, workerImageHash],
      },
      { parent: this, dependsOn: [invocation] },
    );

    this.stageDir = fetchBinary.stdout.apply(() => stageDir);
    this.sourceHash = pulumi.output(sourceHash);

    this.registerOutputs({
      stageDir: this.stageDir,
      sourceHash: this.sourceHash,
    });
  }
}

function hashDir(dir: string): string {
  const hash = crypto.createHash("sha256");
  for (const entry of walk(dir)) {
    hash.update(entry.rel);
    hash.update("\0");
    hash.update(fs.readFileSync(entry.abs));
  }
  return hash.digest("hex").slice(0, 16);
}

function hashSourceDirs(root: string, dirs: string[]): string {
  const hash = crypto.createHash("sha256");
  for (const d of dirs) {
    const base = path.join(root, d);
    for (const entry of walk(base)) {
      if (shouldSkip(entry.rel)) continue;
      hash.update(`${d}/${entry.rel}`);
      hash.update("\0");
      hash.update(fs.readFileSync(entry.abs));
    }
  }
  return hash.digest("hex").slice(0, 16);
}

function shouldSkip(rel: string): boolean {
  return (
    rel.startsWith("target/") ||
    rel.includes("/target/") ||
    rel.endsWith(".DS_Store")
  );
}

function* walk(dir: string): Generator<{ abs: string; rel: string }> {
  const stack: Array<{ abs: string; rel: string }> = [{ abs: dir, rel: "" }];
  while (stack.length) {
    const { abs, rel } = stack.pop()!;
    const entries = fs.readdirSync(abs, { withFileTypes: true }).sort((a, b) =>
      a.name.localeCompare(b.name),
    );
    for (const e of entries) {
      const childAbs = path.join(abs, e.name);
      const childRel = rel ? `${rel}/${e.name}` : e.name;
      if (e.isDirectory()) {
        stack.push({ abs: childAbs, rel: childRel });
      } else if (e.isFile()) {
        yield { abs: childAbs, rel: childRel };
      }
    }
  }
}
