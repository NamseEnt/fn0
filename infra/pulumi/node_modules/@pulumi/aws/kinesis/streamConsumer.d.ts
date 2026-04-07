import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage a Kinesis Stream Consumer.
 *
 * > **Note:** You can register up to 20 consumers per stream. A given consumer can only be registered with one stream at a time.
 *
 * For more details, see the [Amazon Kinesis Stream Consumer Documentation](https://docs.aws.amazon.com/streams/latest/dev/amazon-kinesis-consumers.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kinesis.Stream("example", {
 *     name: "example-stream",
 *     shardCount: 1,
 * });
 * const exampleStreamConsumer = new aws.kinesis.StreamConsumer("example", {
 *     name: "example-consumer",
 *     streamArn: example.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Kinesis Stream Consumers using the Amazon Resource Name (ARN). For example:
 *
 * ```sh
 * $ pulumi import aws:kinesis/streamConsumer:StreamConsumer example arn:aws:kinesis:us-west-2:123456789012:stream/example/consumer/example:1616044553
 * ```
 */
export declare class StreamConsumer extends pulumi.CustomResource {
    /**
     * Get an existing StreamConsumer resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: StreamConsumerState, opts?: pulumi.CustomResourceOptions): StreamConsumer;
    /**
     * Returns true if the given object is an instance of StreamConsumer.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is StreamConsumer;
    /**
     * Amazon Resource Name (ARN) of the stream consumer.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Approximate timestamp in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8) of when the stream consumer was created.
     */
    readonly creationTimestamp: pulumi.Output<string>;
    /**
     * Name of the stream consumer.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the data stream the consumer is registered with.
     */
    readonly streamArn: pulumi.Output<string>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a StreamConsumer resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: StreamConsumerArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering StreamConsumer resources.
 */
export interface StreamConsumerState {
    /**
     * Amazon Resource Name (ARN) of the stream consumer.
     */
    arn?: pulumi.Input<string>;
    /**
     * Approximate timestamp in [RFC3339 format](https://tools.ietf.org/html/rfc3339#section-5.8) of when the stream consumer was created.
     */
    creationTimestamp?: pulumi.Input<string>;
    /**
     * Name of the stream consumer.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the data stream the consumer is registered with.
     */
    streamArn?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a StreamConsumer resource.
 */
export interface StreamConsumerArgs {
    /**
     * Name of the stream consumer.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the data stream the consumer is registered with.
     */
    streamArn: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
