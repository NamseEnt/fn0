"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.AwsWasmS3 = void 0;
const pulumi = require("@pulumi/pulumi");
const aws = require("@pulumi/aws");
class AwsWasmS3 extends pulumi.ComponentResource {
    constructor(name, args, opts) {
        super("pkg:index:s3-build-trigger-queue", name, args, opts);
        const { region } = args;
        const wasmBucket = new aws.s3.Bucket("wasm-bucket", {
            region,
        }, { parent: this });
        this.bucket = wasmBucket.bucket;
        new aws.s3.BucketLifecycleConfiguration("wasm-bucket-lifecycle", {
            region,
            bucket: wasmBucket.bucket,
            rules: [
                {
                    id: "wasm-bucket-lifecycle-rule",
                    status: "Enabled",
                    expiration: {
                        days: 1,
                    },
                },
            ],
        }, { parent: this });
        const queue = new aws.sqs.Queue("queue", {
            region,
        }, { parent: this });
        this.queueArn = queue.arn;
        const queuePolicyDoc = aws.iam.getPolicyDocumentOutput({
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
                            values: [wasmBucket.arn],
                        },
                    ],
                },
            ],
        }, { parent: this });
        const queuePolicy = new aws.sqs.QueuePolicy("allow-s3-send-message", {
            region,
            queueUrl: queue.id,
            policy: queuePolicyDoc.json,
        }, { parent: this });
        new aws.s3.BucketNotification("bucket-notification", {
            region,
            bucket: wasmBucket.bucket,
            queues: [
                {
                    queueArn: queue.arn,
                    events: ["s3:ObjectCreated:*"],
                },
            ],
        }, { parent: this, dependsOn: [queuePolicy] });
    }
}
exports.AwsWasmS3 = AwsWasmS3;
//# sourceMappingURL=AwsWasmS3.js.map