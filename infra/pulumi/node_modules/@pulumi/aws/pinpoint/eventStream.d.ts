import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Pinpoint Event Stream resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const app = new aws.pinpoint.App("app", {});
 * const testStream = new aws.kinesis.Stream("test_stream", {
 *     name: "pinpoint-kinesis-test",
 *     shardCount: 1,
 * });
 * const assumeRole = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["pinpoint.us-east-1.amazonaws.com"],
 *         }],
 *         actions: ["sts:AssumeRole"],
 *     }],
 * });
 * const testRole = new aws.iam.Role("test_role", {assumeRolePolicy: assumeRole.then(assumeRole => assumeRole.json)});
 * const stream = new aws.pinpoint.EventStream("stream", {
 *     applicationId: app.applicationId,
 *     destinationStreamArn: testStream.arn,
 *     roleArn: testRole.arn,
 * });
 * const testRolePolicy = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         actions: [
 *             "kinesis:PutRecords",
 *             "kinesis:DescribeStream",
 *         ],
 *         resources: ["arn:aws:kinesis:us-east-1:*:*&#47;*"],
 *     }],
 * });
 * const testRolePolicyRolePolicy = new aws.iam.RolePolicy("test_role_policy", {
 *     name: "test_policy",
 *     role: testRole.id,
 *     policy: testRolePolicy.then(testRolePolicy => testRolePolicy.json),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Pinpoint Event Stream using the `application-id`. For example:
 *
 * ```sh
 * $ pulumi import aws:pinpoint/eventStream:EventStream stream application-id
 * ```
 */
export declare class EventStream extends pulumi.CustomResource {
    /**
     * Get an existing EventStream resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EventStreamState, opts?: pulumi.CustomResourceOptions): EventStream;
    /**
     * Returns true if the given object is an instance of EventStream.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EventStream;
    /**
     * The application ID.
     */
    readonly applicationId: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the Amazon Kinesis stream or Firehose delivery stream to which you want to publish events.
     */
    readonly destinationStreamArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The IAM role that authorizes Amazon Pinpoint to publish events to the stream in your account.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * Create a EventStream resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EventStreamArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EventStream resources.
 */
export interface EventStreamState {
    /**
     * The application ID.
     */
    applicationId?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the Amazon Kinesis stream or Firehose delivery stream to which you want to publish events.
     */
    destinationStreamArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IAM role that authorizes Amazon Pinpoint to publish events to the stream in your account.
     */
    roleArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EventStream resource.
 */
export interface EventStreamArgs {
    /**
     * The application ID.
     */
    applicationId: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the Amazon Kinesis stream or Firehose delivery stream to which you want to publish events.
     */
    destinationStreamArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IAM role that authorizes Amazon Pinpoint to publish events to the stream in your account.
     */
    roleArn: pulumi.Input<string>;
}
