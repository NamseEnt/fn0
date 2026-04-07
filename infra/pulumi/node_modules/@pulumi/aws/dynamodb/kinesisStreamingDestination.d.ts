import * as pulumi from "@pulumi/pulumi";
/**
 * Enables a [Kinesis streaming destination](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/kds.html) for data replication of a DynamoDB table.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.dynamodb.Table("example", {
 *     name: "orders",
 *     hashKey: "id",
 *     attributes: [{
 *         name: "id",
 *         type: "S",
 *     }],
 * });
 * const exampleStream = new aws.kinesis.Stream("example", {
 *     name: "order_item_changes",
 *     shardCount: 1,
 * });
 * const exampleKinesisStreamingDestination = new aws.dynamodb.KinesisStreamingDestination("example", {
 *     streamArn: exampleStream.arn,
 *     tableName: example.name,
 *     approximateCreationDateTimePrecision: "MICROSECOND",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import DynamoDB Kinesis Streaming Destinations using the `table_name` and `stream_arn` separated by `,`. For example:
 *
 * ```sh
 * $ pulumi import aws:dynamodb/kinesisStreamingDestination:KinesisStreamingDestination example example,arn:aws:kinesis:us-east-1:111122223333:exampleStreamName
 * ```
 */
export declare class KinesisStreamingDestination extends pulumi.CustomResource {
    /**
     * Get an existing KinesisStreamingDestination resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: KinesisStreamingDestinationState, opts?: pulumi.CustomResourceOptions): KinesisStreamingDestination;
    /**
     * Returns true if the given object is an instance of KinesisStreamingDestination.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is KinesisStreamingDestination;
    /**
     * Toggle for the precision of Kinesis data stream timestamp. Valid values: `MILLISECOND` and `MICROSECOND`.
     */
    readonly approximateCreationDateTimePrecision: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ARN for a Kinesis data stream. This must exist in the same account and region as the DynamoDB table.
     */
    readonly streamArn: pulumi.Output<string>;
    /**
     * The name of the DynamoDB table. There can only be one Kinesis streaming destination for a given DynamoDB table.
     */
    readonly tableName: pulumi.Output<string>;
    /**
     * Create a KinesisStreamingDestination resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: KinesisStreamingDestinationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering KinesisStreamingDestination resources.
 */
export interface KinesisStreamingDestinationState {
    /**
     * Toggle for the precision of Kinesis data stream timestamp. Valid values: `MILLISECOND` and `MICROSECOND`.
     */
    approximateCreationDateTimePrecision?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN for a Kinesis data stream. This must exist in the same account and region as the DynamoDB table.
     */
    streamArn?: pulumi.Input<string>;
    /**
     * The name of the DynamoDB table. There can only be one Kinesis streaming destination for a given DynamoDB table.
     */
    tableName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a KinesisStreamingDestination resource.
 */
export interface KinesisStreamingDestinationArgs {
    /**
     * Toggle for the precision of Kinesis data stream timestamp. Valid values: `MILLISECOND` and `MICROSECOND`.
     */
    approximateCreationDateTimePrecision?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARN for a Kinesis data stream. This must exist in the same account and region as the DynamoDB table.
     */
    streamArn: pulumi.Input<string>;
    /**
     * The name of the DynamoDB table. There can only be one Kinesis streaming destination for a given DynamoDB table.
     */
    tableName: pulumi.Input<string>;
}
