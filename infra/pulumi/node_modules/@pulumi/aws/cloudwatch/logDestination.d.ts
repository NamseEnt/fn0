import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a CloudWatch Logs destination resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testDestination = new aws.cloudwatch.LogDestination("test_destination", {
 *     name: "test_destination",
 *     roleArn: iamForCloudwatch.arn,
 *     targetArn: kinesisForCloudwatch.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudWatch Logs destinations using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudwatch/logDestination:LogDestination test_destination test_destination
 * ```
 */
export declare class LogDestination extends pulumi.CustomResource {
    /**
     * Get an existing LogDestination resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LogDestinationState, opts?: pulumi.CustomResourceOptions): LogDestination;
    /**
     * Returns true if the given object is an instance of LogDestination.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LogDestination;
    /**
     * The Amazon Resource Name (ARN) specifying the log destination.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A name for the log destination.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ARN of an IAM role that grants Amazon CloudWatch Logs permissions to put data into the target.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The ARN of the target Amazon Kinesis stream resource for the destination.
     */
    readonly targetArn: pulumi.Output<string>;
    /**
     * Create a LogDestination resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LogDestinationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LogDestination resources.
 */
export interface LogDestinationState {
    /**
     * The Amazon Resource Name (ARN) specifying the log destination.
     */
    arn?: pulumi.Input<string>;
    /**
     * A name for the log destination.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of an IAM role that grants Amazon CloudWatch Logs permissions to put data into the target.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ARN of the target Amazon Kinesis stream resource for the destination.
     */
    targetArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a LogDestination resource.
 */
export interface LogDestinationArgs {
    /**
     * A name for the log destination.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN of an IAM role that grants Amazon CloudWatch Logs permissions to put data into the target.
     */
    roleArn: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ARN of the target Amazon Kinesis stream resource for the destination.
     */
    targetArn: pulumi.Input<string>;
}
