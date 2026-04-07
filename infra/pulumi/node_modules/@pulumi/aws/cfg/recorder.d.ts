import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS Config Configuration Recorder. Please note that this resource **does not start** the created recorder automatically.
 *
 * > **Note:** _Starting_ the Configuration Recorder requires a delivery channel (while delivery channel creation requires Configuration Recorder). This is why `aws.cfg.RecorderStatus` is a separate resource.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
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
 * const foo = new aws.cfg.Recorder("foo", {
 *     name: "example",
 *     roleArn: r.arn,
 * });
 * ```
 *
 * ### Exclude Resources Types Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const foo = new aws.cfg.Recorder("foo", {
 *     name: "example",
 *     roleArn: r.arn,
 *     recordingGroup: {
 *         allSupported: false,
 *         exclusionByResourceTypes: [{
 *             resourceTypes: ["AWS::EC2::Instance"],
 *         }],
 *         recordingStrategies: [{
 *             useOnly: "EXCLUSION_BY_RESOURCE_TYPES",
 *         }],
 *     },
 * });
 * ```
 *
 * ### Periodic Recording
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const foo = new aws.cfg.Recorder("foo", {
 *     name: "example",
 *     roleArn: r.arn,
 *     recordingGroup: {
 *         allSupported: false,
 *         includeGlobalResourceTypes: false,
 *         resourceTypes: [
 *             "AWS::EC2::Instance",
 *             "AWS::EC2::NetworkInterface",
 *         ],
 *     },
 *     recordingMode: {
 *         recordingFrequency: "CONTINUOUS",
 *         recordingModeOverride: {
 *             description: "Only record EC2 network interfaces daily",
 *             resourceTypes: ["AWS::EC2::NetworkInterface"],
 *             recordingFrequency: "DAILY",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Configuration Recorder using the name. For example:
 *
 * ```sh
 * $ pulumi import aws:cfg/recorder:Recorder foo example
 * ```
 */
export declare class Recorder extends pulumi.CustomResource {
    /**
     * Get an existing Recorder resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RecorderState, opts?: pulumi.CustomResourceOptions): Recorder;
    /**
     * Returns true if the given object is an instance of Recorder.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Recorder;
    /**
     * The name of the recorder. Defaults to `default`. Changing it recreates the resource.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Recording group - see below.
     */
    readonly recordingGroup: pulumi.Output<outputs.cfg.RecorderRecordingGroup>;
    /**
     * Recording mode - see below.
     */
    readonly recordingMode: pulumi.Output<outputs.cfg.RecorderRecordingMode>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the IAM role. Used to make read or write requests to the delivery channel and to describe the AWS resources associated with the account. See [AWS Docs](http://docs.aws.amazon.com/config/latest/developerguide/iamrole-permissions.html) for more details.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * Create a Recorder resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RecorderArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Recorder resources.
 */
export interface RecorderState {
    /**
     * The name of the recorder. Defaults to `default`. Changing it recreates the resource.
     */
    name?: pulumi.Input<string>;
    /**
     * Recording group - see below.
     */
    recordingGroup?: pulumi.Input<inputs.cfg.RecorderRecordingGroup>;
    /**
     * Recording mode - see below.
     */
    recordingMode?: pulumi.Input<inputs.cfg.RecorderRecordingMode>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the IAM role. Used to make read or write requests to the delivery channel and to describe the AWS resources associated with the account. See [AWS Docs](http://docs.aws.amazon.com/config/latest/developerguide/iamrole-permissions.html) for more details.
     */
    roleArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Recorder resource.
 */
export interface RecorderArgs {
    /**
     * The name of the recorder. Defaults to `default`. Changing it recreates the resource.
     */
    name?: pulumi.Input<string>;
    /**
     * Recording group - see below.
     */
    recordingGroup?: pulumi.Input<inputs.cfg.RecorderRecordingGroup>;
    /**
     * Recording mode - see below.
     */
    recordingMode?: pulumi.Input<inputs.cfg.RecorderRecordingMode>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the IAM role. Used to make read or write requests to the delivery channel and to describe the AWS resources associated with the account. See [AWS Docs](http://docs.aws.amazon.com/config/latest/developerguide/iamrole-permissions.html) for more details.
     */
    roleArn: pulumi.Input<string>;
}
