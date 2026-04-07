import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS RDS (Relational Database) Export Task.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.rds.ExportTask("example", {
 *     exportTaskIdentifier: "example",
 *     sourceArn: exampleAwsDbSnapshot.dbSnapshotArn,
 *     s3BucketName: exampleAwsS3Bucket.id,
 *     iamRoleArn: exampleAwsIamRole.arn,
 *     kmsKeyId: exampleAwsKmsKey.arn,
 * });
 * ```
 *
 * ### Complete Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleBucket = new aws.s3.Bucket("example", {
 *     bucket: "example",
 *     forceDestroy: true,
 * });
 * const exampleBucketAcl = new aws.s3.BucketAcl("example", {
 *     bucket: exampleBucket.id,
 *     acl: "private",
 * });
 * const exampleRole = new aws.iam.Role("example", {
 *     name: "example",
 *     assumeRolePolicy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Action: "sts:AssumeRole",
 *             Effect: "Allow",
 *             Sid: "",
 *             Principal: {
 *                 Service: "export.rds.amazonaws.com",
 *             },
 *         }],
 *     }),
 * });
 * const example = aws.iam.getPolicyDocumentOutput({
 *     statements: [
 *         {
 *             actions: ["s3:ListAllMyBuckets"],
 *             resources: ["*"],
 *         },
 *         {
 *             actions: [
 *                 "s3:GetBucketLocation",
 *                 "s3:ListBucket",
 *             ],
 *             resources: [exampleBucket.arn],
 *         },
 *         {
 *             actions: [
 *                 "s3:GetObject",
 *                 "s3:PutObject",
 *                 "s3:DeleteObject",
 *             ],
 *             resources: [pulumi.interpolate`${exampleBucket.arn}/*`],
 *         },
 *     ],
 * });
 * const examplePolicy = new aws.iam.Policy("example", {
 *     name: "example",
 *     policy: example.apply(example => example.json),
 * });
 * const exampleRolePolicyAttachment = new aws.iam.RolePolicyAttachment("example", {
 *     role: exampleRole.name,
 *     policyArn: examplePolicy.arn,
 * });
 * const exampleKey = new aws.kms.Key("example", {deletionWindowInDays: 10});
 * const exampleInstance = new aws.rds.Instance("example", {
 *     identifier: "example",
 *     allocatedStorage: 10,
 *     dbName: "test",
 *     engine: "mysql",
 *     engineVersion: "5.7",
 *     instanceClass: aws.rds.InstanceType.T3_Micro,
 *     username: "foo",
 *     password: "foobarbaz",
 *     parameterGroupName: "default.mysql5.7",
 *     skipFinalSnapshot: true,
 * });
 * const exampleSnapshot = new aws.rds.Snapshot("example", {
 *     dbInstanceIdentifier: exampleInstance.identifier,
 *     dbSnapshotIdentifier: "example",
 * });
 * const exampleExportTask = new aws.rds.ExportTask("example", {
 *     exportTaskIdentifier: "example",
 *     sourceArn: exampleSnapshot.dbSnapshotArn,
 *     s3BucketName: exampleBucket.id,
 *     iamRoleArn: exampleRole.arn,
 *     kmsKeyId: exampleKey.arn,
 *     exportOnlies: ["database"],
 *     s3Prefix: "my_prefix/example",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a RDS (Relational Database) Export Task using the `export_task_identifier`. For example:
 *
 * ```sh
 * $ pulumi import aws:rds/exportTask:ExportTask example example
 * ```
 */
export declare class ExportTask extends pulumi.CustomResource {
    /**
     * Get an existing ExportTask resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ExportTaskState, opts?: pulumi.CustomResourceOptions): ExportTask;
    /**
     * Returns true if the given object is an instance of ExportTask.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ExportTask;
    /**
     * Data to be exported from the snapshot. If this parameter is not provided, all the snapshot data is exported. Valid values are documented in the [AWS StartExportTask API documentation](https://docs.aws.amazon.com/AmazonRDS/latest/APIReference/API_StartExportTask.html#API_StartExportTask_RequestParameters).
     */
    readonly exportOnlies: pulumi.Output<string[] | undefined>;
    /**
     * Unique identifier for the snapshot export task.
     */
    readonly exportTaskIdentifier: pulumi.Output<string>;
    /**
     * Reason the export failed, if it failed.
     */
    readonly failureCause: pulumi.Output<string>;
    /**
     * ARN of the IAM role to use for writing to the Amazon S3 bucket.
     */
    readonly iamRoleArn: pulumi.Output<string>;
    /**
     * ID of the Amazon Web Services KMS key to use to encrypt the snapshot.
     */
    readonly kmsKeyId: pulumi.Output<string>;
    /**
     * Progress of the snapshot export task as a percentage.
     */
    readonly percentProgress: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name of the Amazon S3 bucket to export the snapshot to.
     */
    readonly s3BucketName: pulumi.Output<string>;
    /**
     * Amazon S3 bucket prefix to use as the file name and path of the exported snapshot.
     */
    readonly s3Prefix: pulumi.Output<string>;
    /**
     * Time that the snapshot was created.
     */
    readonly snapshotTime: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the snapshot to export.
     *
     * The following arguments are optional:
     */
    readonly sourceArn: pulumi.Output<string>;
    /**
     * Type of source for the export.
     */
    readonly sourceType: pulumi.Output<string>;
    /**
     * Status of the export task.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Time that the snapshot export task completed.
     */
    readonly taskEndTime: pulumi.Output<string>;
    /**
     * Time that the snapshot export task started.
     */
    readonly taskStartTime: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.rds.ExportTaskTimeouts | undefined>;
    /**
     * Warning about the snapshot export task, if any.
     */
    readonly warningMessage: pulumi.Output<string>;
    /**
     * Create a ExportTask resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ExportTaskArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ExportTask resources.
 */
export interface ExportTaskState {
    /**
     * Data to be exported from the snapshot. If this parameter is not provided, all the snapshot data is exported. Valid values are documented in the [AWS StartExportTask API documentation](https://docs.aws.amazon.com/AmazonRDS/latest/APIReference/API_StartExportTask.html#API_StartExportTask_RequestParameters).
     */
    exportOnlies?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Unique identifier for the snapshot export task.
     */
    exportTaskIdentifier?: pulumi.Input<string>;
    /**
     * Reason the export failed, if it failed.
     */
    failureCause?: pulumi.Input<string>;
    /**
     * ARN of the IAM role to use for writing to the Amazon S3 bucket.
     */
    iamRoleArn?: pulumi.Input<string>;
    /**
     * ID of the Amazon Web Services KMS key to use to encrypt the snapshot.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Progress of the snapshot export task as a percentage.
     */
    percentProgress?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the Amazon S3 bucket to export the snapshot to.
     */
    s3BucketName?: pulumi.Input<string>;
    /**
     * Amazon S3 bucket prefix to use as the file name and path of the exported snapshot.
     */
    s3Prefix?: pulumi.Input<string>;
    /**
     * Time that the snapshot was created.
     */
    snapshotTime?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the snapshot to export.
     *
     * The following arguments are optional:
     */
    sourceArn?: pulumi.Input<string>;
    /**
     * Type of source for the export.
     */
    sourceType?: pulumi.Input<string>;
    /**
     * Status of the export task.
     */
    status?: pulumi.Input<string>;
    /**
     * Time that the snapshot export task completed.
     */
    taskEndTime?: pulumi.Input<string>;
    /**
     * Time that the snapshot export task started.
     */
    taskStartTime?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.rds.ExportTaskTimeouts>;
    /**
     * Warning about the snapshot export task, if any.
     */
    warningMessage?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ExportTask resource.
 */
export interface ExportTaskArgs {
    /**
     * Data to be exported from the snapshot. If this parameter is not provided, all the snapshot data is exported. Valid values are documented in the [AWS StartExportTask API documentation](https://docs.aws.amazon.com/AmazonRDS/latest/APIReference/API_StartExportTask.html#API_StartExportTask_RequestParameters).
     */
    exportOnlies?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Unique identifier for the snapshot export task.
     */
    exportTaskIdentifier: pulumi.Input<string>;
    /**
     * ARN of the IAM role to use for writing to the Amazon S3 bucket.
     */
    iamRoleArn: pulumi.Input<string>;
    /**
     * ID of the Amazon Web Services KMS key to use to encrypt the snapshot.
     */
    kmsKeyId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the Amazon S3 bucket to export the snapshot to.
     */
    s3BucketName: pulumi.Input<string>;
    /**
     * Amazon S3 bucket prefix to use as the file name and path of the exported snapshot.
     */
    s3Prefix?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the snapshot to export.
     *
     * The following arguments are optional:
     */
    sourceArn: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.rds.ExportTaskTimeouts>;
}
