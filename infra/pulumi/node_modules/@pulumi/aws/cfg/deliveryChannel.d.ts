import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS Config Delivery Channel.
 *
 * > **Note:** Delivery Channel requires a Configuration Recorder to be present. Use of `dependsOn` (as shown below) is recommended to avoid race conditions.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const b = new aws.s3.Bucket("b", {
 *     bucket: "example-awsconfig",
 *     forceDestroy: true,
 * });
 * const assumeRole = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["config.amazonaws.com"],
 *         }],
 *         actions: ["sts:AssumeRole"],
 *     }],
 * });
 * const r = new aws.iam.Role("r", {
 *     name: "awsconfig-example",
 *     assumeRolePolicy: assumeRole.then(assumeRole => assumeRole.json),
 * });
 * const fooRecorder = new aws.cfg.Recorder("foo", {
 *     name: "example",
 *     roleArn: r.arn,
 * });
 * const foo = new aws.cfg.DeliveryChannel("foo", {
 *     name: "example",
 *     s3BucketName: b.bucket,
 * }, {
 *     dependsOn: [fooRecorder],
 * });
 * const p = aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         effect: "Allow",
 *         actions: ["s3:*"],
 *         resources: [
 *             b.arn,
 *             pulumi.interpolate`${b.arn}/*`,
 *         ],
 *     }],
 * });
 * const pRolePolicy = new aws.iam.RolePolicy("p", {
 *     name: "awsconfig-example",
 *     role: r.id,
 *     policy: p.apply(p => p.json),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Delivery Channel using the name. For example:
 *
 * ```sh
 * $ pulumi import aws:cfg/deliveryChannel:DeliveryChannel foo example
 * ```
 */
export declare class DeliveryChannel extends pulumi.CustomResource {
    /**
     * Get an existing DeliveryChannel resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DeliveryChannelState, opts?: pulumi.CustomResourceOptions): DeliveryChannel;
    /**
     * Returns true if the given object is an instance of DeliveryChannel.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DeliveryChannel;
    /**
     * The name of the delivery channel. Defaults to `default`. Changing it recreates the resource.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The name of the S3 bucket used to store the configuration history.
     */
    readonly s3BucketName: pulumi.Output<string>;
    /**
     * The prefix for the specified S3 bucket.
     */
    readonly s3KeyPrefix: pulumi.Output<string | undefined>;
    /**
     * The ARN of the AWS KMS key used to encrypt objects delivered by AWS Config. Must belong to the same Region as the destination S3 bucket.
     */
    readonly s3KmsKeyArn: pulumi.Output<string | undefined>;
    /**
     * Options for how AWS Config delivers configuration snapshots. See below
     */
    readonly snapshotDeliveryProperties: pulumi.Output<outputs.cfg.DeliveryChannelSnapshotDeliveryProperties | undefined>;
    /**
     * The ARN of the SNS topic that AWS Config delivers notifications to.
     */
    readonly snsTopicArn: pulumi.Output<string | undefined>;
    /**
     * Create a DeliveryChannel resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DeliveryChannelArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DeliveryChannel resources.
 */
export interface DeliveryChannelState {
    /**
     * The name of the delivery channel. Defaults to `default`. Changing it recreates the resource.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the S3 bucket used to store the configuration history.
     */
    s3BucketName?: pulumi.Input<string>;
    /**
     * The prefix for the specified S3 bucket.
     */
    s3KeyPrefix?: pulumi.Input<string>;
    /**
     * The ARN of the AWS KMS key used to encrypt objects delivered by AWS Config. Must belong to the same Region as the destination S3 bucket.
     */
    s3KmsKeyArn?: pulumi.Input<string>;
    /**
     * Options for how AWS Config delivers configuration snapshots. See below
     */
    snapshotDeliveryProperties?: pulumi.Input<inputs.cfg.DeliveryChannelSnapshotDeliveryProperties>;
    /**
     * The ARN of the SNS topic that AWS Config delivers notifications to.
     */
    snsTopicArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DeliveryChannel resource.
 */
export interface DeliveryChannelArgs {
    /**
     * The name of the delivery channel. Defaults to `default`. Changing it recreates the resource.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the S3 bucket used to store the configuration history.
     */
    s3BucketName: pulumi.Input<string>;
    /**
     * The prefix for the specified S3 bucket.
     */
    s3KeyPrefix?: pulumi.Input<string>;
    /**
     * The ARN of the AWS KMS key used to encrypt objects delivered by AWS Config. Must belong to the same Region as the destination S3 bucket.
     */
    s3KmsKeyArn?: pulumi.Input<string>;
    /**
     * Options for how AWS Config delivers configuration snapshots. See below
     */
    snapshotDeliveryProperties?: pulumi.Input<inputs.cfg.DeliveryChannelSnapshotDeliveryProperties>;
    /**
     * The ARN of the SNS topic that AWS Config delivers notifications to.
     */
    snsTopicArn?: pulumi.Input<string>;
}
