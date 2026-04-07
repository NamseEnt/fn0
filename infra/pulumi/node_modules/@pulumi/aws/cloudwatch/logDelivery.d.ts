import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS CloudWatch Logs Delivery. A delivery is a connection between an `aws.cloudwatch.LogDeliverySource` and an `aws.cloudwatch.LogDeliveryDestination`.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloudwatch.LogDelivery("example", {
 *     deliverySourceName: exampleAwsCloudwatchLogDeliverySource.name,
 *     deliveryDestinationArn: exampleAwsCloudwatchLogDeliveryDestination.arn,
 *     fieldDelimiter: ",",
 *     recordFields: [
 *         "event_timestamp",
 *         "event",
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudWatch Logs Delivery using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudwatch/logDelivery:LogDelivery example jsoGVi4Zq8VlYp9n
 * ```
 */
export declare class LogDelivery extends pulumi.CustomResource {
    /**
     * Get an existing LogDelivery resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LogDeliveryState, opts?: pulumi.CustomResourceOptions): LogDelivery;
    /**
     * Returns true if the given object is an instance of LogDelivery.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LogDelivery;
    /**
     * The Amazon Resource Name (ARN) of the delivery.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The ARN of the delivery destination to use for this delivery.
     */
    readonly deliveryDestinationArn: pulumi.Output<string>;
    /**
     * The name of the delivery source to use for this delivery.
     */
    readonly deliverySourceName: pulumi.Output<string>;
    /**
     * The field delimiter to use between record fields when the final output format of a delivery is in `plain`, `w3c`, or `raw` format.
     */
    readonly fieldDelimiter: pulumi.Output<string>;
    /**
     * The list of record fields to be delivered to the destination, in order.
     */
    readonly recordFields: pulumi.Output<string[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Parameters that are valid only when the delivery's delivery destination is an S3 bucket.
     */
    readonly s3DeliveryConfigurations: pulumi.Output<outputs.cloudwatch.LogDeliveryS3DeliveryConfiguration[]>;
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
     * Create a LogDelivery resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: LogDeliveryArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LogDelivery resources.
 */
export interface LogDeliveryState {
    /**
     * The Amazon Resource Name (ARN) of the delivery.
     */
    arn?: pulumi.Input<string>;
    /**
     * The ARN of the delivery destination to use for this delivery.
     */
    deliveryDestinationArn?: pulumi.Input<string>;
    /**
     * The name of the delivery source to use for this delivery.
     */
    deliverySourceName?: pulumi.Input<string>;
    /**
     * The field delimiter to use between record fields when the final output format of a delivery is in `plain`, `w3c`, or `raw` format.
     */
    fieldDelimiter?: pulumi.Input<string>;
    /**
     * The list of record fields to be delivered to the destination, in order.
     */
    recordFields?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Parameters that are valid only when the delivery's delivery destination is an S3 bucket.
     */
    s3DeliveryConfigurations?: pulumi.Input<pulumi.Input<inputs.cloudwatch.LogDeliveryS3DeliveryConfiguration>[]>;
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
 * The set of arguments for constructing a LogDelivery resource.
 */
export interface LogDeliveryArgs {
    /**
     * The ARN of the delivery destination to use for this delivery.
     */
    deliveryDestinationArn: pulumi.Input<string>;
    /**
     * The name of the delivery source to use for this delivery.
     */
    deliverySourceName: pulumi.Input<string>;
    /**
     * The field delimiter to use between record fields when the final output format of a delivery is in `plain`, `w3c`, or `raw` format.
     */
    fieldDelimiter?: pulumi.Input<string>;
    /**
     * The list of record fields to be delivered to the destination, in order.
     */
    recordFields?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Parameters that are valid only when the delivery's delivery destination is an S3 bucket.
     */
    s3DeliveryConfigurations?: pulumi.Input<pulumi.Input<inputs.cloudwatch.LogDeliveryS3DeliveryConfiguration>[]>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
