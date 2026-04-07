import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS CloudWatch Logs Delivery Destination.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudwatch.LogDeliveryDestination("example", {
 *     name: "example",
 *     deliveryDestinationConfiguration: {
 *         destinationResourceArn: exampleAwsCloudwatchLogGroup.arn,
 *     },
 * });
 * ```
 *
 * ### X-Ray Trace Delivery
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const xray = new aws.cloudwatch.LogDeliveryDestination("xray", {
 *     name: "xray-traces",
 *     deliveryDestinationType: "XRAY",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudWatch Logs Delivery Destination using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudwatch/logDeliveryDestination:LogDeliveryDestination example example
 * ```
 */
export declare class LogDeliveryDestination extends pulumi.CustomResource {
    /**
     * Get an existing LogDeliveryDestination resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LogDeliveryDestinationState, opts?: pulumi.CustomResourceOptions): LogDeliveryDestination;
    /**
     * Returns true if the given object is an instance of LogDeliveryDestination.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LogDeliveryDestination;
    /**
     * The Amazon Resource Name (ARN) of the delivery destination.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The AWS resource that will receive the logs. Required for CloudWatch Logs, Amazon S3, and Firehose destinations. Not required for X-Ray trace delivery destinations.
     */
    readonly deliveryDestinationConfiguration: pulumi.Output<outputs.cloudwatch.LogDeliveryDestinationDeliveryDestinationConfiguration | undefined>;
    /**
     * The type of delivery destination. Valid values: `S3`, `CWL`, `FH`, `XRAY`. Required for X-Ray trace delivery destinations. For other destination types, this is computed from the `destinationResourceArn`.
     */
    readonly deliveryDestinationType: pulumi.Output<string>;
    /**
     * The name for this delivery destination.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The format of the logs that are sent to this delivery destination. Valid values: `json`, `plain`, `w3c`, `raw`, `parquet`.
     */
    readonly outputFormat: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
     * Create a LogDeliveryDestination resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: LogDeliveryDestinationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LogDeliveryDestination resources.
 */
export interface LogDeliveryDestinationState {
    /**
     * The Amazon Resource Name (ARN) of the delivery destination.
     */
    arn?: pulumi.Input<string>;
    /**
     * The AWS resource that will receive the logs. Required for CloudWatch Logs, Amazon S3, and Firehose destinations. Not required for X-Ray trace delivery destinations.
     */
    deliveryDestinationConfiguration?: pulumi.Input<inputs.cloudwatch.LogDeliveryDestinationDeliveryDestinationConfiguration>;
    /**
     * The type of delivery destination. Valid values: `S3`, `CWL`, `FH`, `XRAY`. Required for X-Ray trace delivery destinations. For other destination types, this is computed from the `destinationResourceArn`.
     */
    deliveryDestinationType?: pulumi.Input<string>;
    /**
     * The name for this delivery destination.
     */
    name?: pulumi.Input<string>;
    /**
     * The format of the logs that are sent to this delivery destination. Valid values: `json`, `plain`, `w3c`, `raw`, `parquet`.
     */
    outputFormat?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
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
}
/**
 * The set of arguments for constructing a LogDeliveryDestination resource.
 */
export interface LogDeliveryDestinationArgs {
    /**
     * The AWS resource that will receive the logs. Required for CloudWatch Logs, Amazon S3, and Firehose destinations. Not required for X-Ray trace delivery destinations.
     */
    deliveryDestinationConfiguration?: pulumi.Input<inputs.cloudwatch.LogDeliveryDestinationDeliveryDestinationConfiguration>;
    /**
     * The type of delivery destination. Valid values: `S3`, `CWL`, `FH`, `XRAY`. Required for X-Ray trace delivery destinations. For other destination types, this is computed from the `destinationResourceArn`.
     */
    deliveryDestinationType?: pulumi.Input<string>;
    /**
     * The name for this delivery destination.
     */
    name?: pulumi.Input<string>;
    /**
     * The format of the logs that are sent to this delivery destination. Valid values: `json`, `plain`, `w3c`, `raw`, `parquet`.
     */
    outputFormat?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
