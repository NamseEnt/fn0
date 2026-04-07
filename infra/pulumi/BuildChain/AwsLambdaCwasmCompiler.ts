import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";
import * as docker from "@pulumi/docker";
import * as command from "@pulumi/command";
import * as path from "node:path";

export interface AwsLambdaCwasmCompilerArgs {
  region: pulumi.Input<string>;
  wasmBucket: pulumi.Input<string>;
  cWasmBucket: pulumi.Input<string>;
  queueArn: pulumi.Input<string>;
}

export class AwsLambdaCwasmCompiler extends pulumi.ComponentResource {
  constructor(
    name: string,
    args: AwsLambdaCwasmCompilerArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:aws-lambda-cwasm-compiler", name, args, opts);

    const { region, wasmBucket, queueArn, cWasmBucket } = args;

    const fn0CompilerImage = new docker.Image(
      "fn0-wasmtime-image",
      {
        imageName: "fn0-wasmtime:latest",
        build: {
          context: "../..",
          dockerfile: "../../fn0-wasmtime/Dockerfile",
          platform: "linux/amd64",
        },
        skipPush: true,
      },
      { parent: this }
    );

    const fn0CompilerLayerZip = path.join(__dirname, "./fn0-wasmtime-layer.zip");

    const extractBinary = new command.local.Command(
      "extract-fn0-compiler",
      {
        create: pulumi.interpolate`TMPDIR=$(mktemp -d) && docker create --name fn0-wasmtime-extract ${fn0CompilerImage.imageName} /bin/true && docker cp fn0-wasmtime-extract:/fn0-wasmtime $TMPDIR/fn0-wasmtime && docker rm fn0-wasmtime-extract && chmod +x $TMPDIR/fn0-wasmtime && cd $TMPDIR && zip ${fn0CompilerLayerZip} fn0-compiler && rm -rf $TMPDIR`,
        triggers: [fn0CompilerImage.repoDigest],
      },
      { parent: this }
    );

    const fn0CompilerLayer = new aws.lambda.LayerVersion(
      "fn0-wasmtime-layer",
      {
        region,
        layerName: "fn0-compiler",
        code: extractBinary.stdout.apply(
          () => new pulumi.asset.FileArchive(fn0CompilerLayerZip)
        ),
      },
      { parent: this, dependsOn: [extractBinary] }
    );

    const zstdLayer = new aws.lambda.LayerVersion(
      "zstd-layer",
      {
        region,
        layerName: "zstd",
        code: new pulumi.asset.FileArchive(
          path.join(__dirname, "./zstd-layer")
        ),
      },
      { parent: this }
    );

    const wasmBucketArn = pulumi.interpolate`arn:aws:s3:::${wasmBucket}`;
    const cWasmBucketArn = pulumi.interpolate`arn:aws:s3:::${cWasmBucket}`;

    const lambdaRole = new aws.iam.Role("lambda-role", {
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
              name: "s3-access-policy",
              policy: pulumi
                .all([wasmBucketArn, cWasmBucketArn])
                .apply(([wasmBucketArn, cWasmBucketArn]) =>
                  JSON.stringify({
                    Version: "2012-10-17",
                    Statement: [
                      {
                        Effect: "Allow",
                        Action: ["s3:GetObject", "s3:DeleteObject"],
                        Resource: `${wasmBucketArn}/*`,
                      },
                      {
                        Effect: "Allow",
                        Action: ["s3:ListBucket"],
                        Resource: wasmBucketArn,
                      },
                      {
                        Effect: "Allow",
                        Action: ["s3:PutObject"],
                        Resource: `${cWasmBucketArn}/*`,
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

    const lambdaArchive = new pulumi.asset.AssetArchive({
      "index.mjs": new pulumi.asset.FileAsset(
        path.join(__dirname, "cwasm-compiler-lambda/index.mjs")
      ),
    });

    const lambda = new aws.lambda.Function(
      "cwasm-compiler-lambda",
      {
        region,
        runtime: "nodejs24.x",
        handler: "index.handler",
        architectures: ["x86_64"],
        layers: [fn0CompilerLayer.arn, zstdLayer.arn],
        timeout: 20,
        memorySize: 10240,
        role: lambdaRole.arn,
        code: lambdaArchive,
        environment: {
          variables: {
            CWASM_BUCKET: cWasmBucket,
            XDG_CACHE_HOME: "/tmp",
          },
        },
      },
      { parent: this }
    );

    new aws.lambda.EventSourceMapping(
      "sqs-trigger",
      {
        region,
        eventSourceArn: queueArn,
        functionName: lambda.name,
        batchSize: 1,
      },
      { parent: this }
    );
  }
}
