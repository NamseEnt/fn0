import * as pulumi from "@pulumi/pulumi";
import * as cloudflare from "@pulumi/cloudflare";
import * as fs from "fs";
import * as path from "path";

export interface ControlR2WorkerArgs {
  accountId: pulumi.Input<string>;
  scriptName: pulumi.Input<string>;
  bucketName: pulumi.Input<string>;
  queueId: pulumi.Input<string>;
  controlUrl: pulumi.Input<string>;
  adminToken: pulumi.Input<string>;
}

const COMPATIBILITY_DATE = "2026-04-01";
const WORKER_SOURCE_PATH = path.resolve(
  __dirname,
  "../r2-worker/src/index.mjs"
);

export class ControlR2Worker extends pulumi.ComponentResource {
  public readonly scriptName: pulumi.Output<string>;

  constructor(
    name: string,
    args: ControlR2WorkerArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    super("pkg:index:control-r2-worker", name, args, opts);

    const workerSource = fs.readFileSync(WORKER_SOURCE_PATH, "utf-8");

    const script = new cloudflare.WorkersScript(
      "script",
      {
        accountId: args.accountId,
        scriptName: args.scriptName,
        compatibilityDate: COMPATIBILITY_DATE,
        compatibilityFlags: ["nodejs_compat"],
        mainModule: "index.mjs",
        content: workerSource,
        bindings: [
          {
            name: "BUCKET",
            type: "r2_bucket",
            bucketName: args.bucketName,
          },
          {
            name: "CONTROL_URL",
            type: "plain_text",
            text: args.controlUrl,
          },
          {
            name: "CONTROL_ADMIN_TOKEN",
            type: "secret_text",
            text: args.adminToken,
          },
        ],
      },
      { parent: this }
    );

    new cloudflare.QueueConsumer(
      "consumer",
      {
        accountId: args.accountId,
        queueId: args.queueId,
        scriptName: script.scriptName,
        type: "worker",
        settings: {
          batchSize: 10,
          maxConcurrency: 4,
          maxRetries: 5,
          maxWaitTimeMs: 5000,
        },
      },
      { parent: this, dependsOn: [script] }
    );

    this.scriptName = script.scriptName;
  }
}
