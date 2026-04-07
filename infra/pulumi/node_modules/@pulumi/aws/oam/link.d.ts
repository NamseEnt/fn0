import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS CloudWatch Observability Access Manager Link.
 *
 * > **NOTE:** Creating an `aws.oam.Link` may sometimes fail if the `aws.oam.SinkPolicy` for the attached `aws.oam.Sink` is not created before the `aws.oam.Link`. To prevent this, declare an explicit dependency using a `dependsOn` meta-argument.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleSink = new aws.oam.Sink("example", {});
 * const exampleSinkPolicy = new aws.oam.SinkPolicy("example", {sinkIdentifier: exampleSink.arn});
 * const example = new aws.oam.Link("example", {
 *     labelTemplate: "$AccountName",
 *     resourceTypes: ["AWS::CloudWatch::Metric"],
 *     sinkIdentifier: exampleSink.arn,
 *     tags: {
 *         Env: "prod",
 *     },
 * }, {
 *     dependsOn: [exampleSinkPolicy],
 * });
 * ```
 *
 * ### Log Group Filtering
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.oam.Link("example", {
 *     labelTemplate: "$AccountName",
 *     linkConfiguration: {
 *         logGroupConfiguration: {
 *             filter: "LogGroupName LIKE 'aws/lambda/%' OR LogGroupName LIKE 'AWSLogs%'",
 *         },
 *     },
 *     resourceTypes: ["AWS::Logs::LogGroup"],
 *     sinkIdentifier: exampleAwsOamSink.arn,
 * }, {
 *     dependsOn: [exampleAwsOamSinkPolicy],
 * });
 * ```
 *
 * ### Metric Filtering
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.oam.Link("example", {
 *     labelTemplate: "$AccountName",
 *     linkConfiguration: {
 *         metricConfiguration: {
 *             filter: "Namespace IN ('AWS/EC2', 'AWS/ELB', 'AWS/S3')",
 *         },
 *     },
 *     resourceTypes: ["AWS::CloudWatch::Metric"],
 *     sinkIdentifier: exampleAwsOamSink.arn,
 * }, {
 *     dependsOn: [exampleAwsOamSinkPolicy],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudWatch Observability Access Manager Link using the `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:oam/link:Link example arn:aws:oam:us-west-2:123456789012:link/link-id
 * ```
 */
export declare class Link extends pulumi.CustomResource {
    /**
     * Get an existing Link resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LinkState, opts?: pulumi.CustomResourceOptions): Link;
    /**
     * Returns true if the given object is an instance of Link.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Link;
    /**
     * ARN of the link.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Label that is assigned to this link.
     */
    readonly label: pulumi.Output<string>;
    /**
     * Human-readable name to use to identify this source account when you are viewing data from it in the monitoring account.
     */
    readonly labelTemplate: pulumi.Output<string>;
    /**
     * Configuration for creating filters that specify that only some metric namespaces or log groups are to be shared from the source account to the monitoring account. See `linkConfiguration` Block for details.
     */
    readonly linkConfiguration: pulumi.Output<outputs.oam.LinkLinkConfiguration | undefined>;
    /**
     * ID string that AWS generated as part of the link ARN.
     */
    readonly linkId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Types of data that the source account shares with the monitoring account.
     */
    readonly resourceTypes: pulumi.Output<string[]>;
    /**
     * ARN of the sink that is used for this link.
     */
    readonly sinkArn: pulumi.Output<string>;
    /**
     * Identifier of the sink to use to create this link.
     *
     * The following arguments are optional:
     */
    readonly sinkIdentifier: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a Link resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LinkArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Link resources.
 */
export interface LinkState {
    /**
     * ARN of the link.
     */
    arn?: pulumi.Input<string>;
    /**
     * Label that is assigned to this link.
     */
    label?: pulumi.Input<string>;
    /**
     * Human-readable name to use to identify this source account when you are viewing data from it in the monitoring account.
     */
    labelTemplate?: pulumi.Input<string>;
    /**
     * Configuration for creating filters that specify that only some metric namespaces or log groups are to be shared from the source account to the monitoring account. See `linkConfiguration` Block for details.
     */
    linkConfiguration?: pulumi.Input<inputs.oam.LinkLinkConfiguration>;
    /**
     * ID string that AWS generated as part of the link ARN.
     */
    linkId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Types of data that the source account shares with the monitoring account.
     */
    resourceTypes?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * ARN of the sink that is used for this link.
     */
    sinkArn?: pulumi.Input<string>;
    /**
     * Identifier of the sink to use to create this link.
     *
     * The following arguments are optional:
     */
    sinkIdentifier?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a Link resource.
 */
export interface LinkArgs {
    /**
     * Human-readable name to use to identify this source account when you are viewing data from it in the monitoring account.
     */
    labelTemplate: pulumi.Input<string>;
    /**
     * Configuration for creating filters that specify that only some metric namespaces or log groups are to be shared from the source account to the monitoring account. See `linkConfiguration` Block for details.
     */
    linkConfiguration?: pulumi.Input<inputs.oam.LinkLinkConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Types of data that the source account shares with the monitoring account.
     */
    resourceTypes: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Identifier of the sink to use to create this link.
     *
     * The following arguments are optional:
     */
    sinkIdentifier: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
