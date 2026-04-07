import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS AppFabric Ingestion Destination.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appfabric.IngestionDestination("example", {
 *     appBundleArn: exampleAwsAppfabricAppBundle.arn,
 *     ingestionArn: exampleAwsAppfabricIngestion.arn,
 *     processingConfiguration: {
 *         auditLog: {
 *             format: "json",
 *             schema: "raw",
 *         },
 *     },
 *     destinationConfiguration: {
 *         auditLog: {
 *             destination: {
 *                 s3Bucket: {
 *                     bucketName: exampleAwsS3Bucket.bucket,
 *                 },
 *             },
 *         },
 *     },
 * });
 * ```
 */
export declare class IngestionDestination extends pulumi.CustomResource {
    /**
     * Get an existing IngestionDestination resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: IngestionDestinationState, opts?: pulumi.CustomResourceOptions): IngestionDestination;
    /**
     * Returns true if the given object is an instance of IngestionDestination.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is IngestionDestination;
    /**
     * The Amazon Resource Name (ARN) of the app bundle to use for the request.
     */
    readonly appBundleArn: pulumi.Output<string>;
    /**
     * ARN of the Ingestion Destination.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Contains information about the destination of ingested data.
     */
    readonly destinationConfiguration: pulumi.Output<outputs.appfabric.IngestionDestinationDestinationConfiguration | undefined>;
    /**
     * The Amazon Resource Name (ARN) of the ingestion to use for the request.
     */
    readonly ingestionArn: pulumi.Output<string>;
    /**
     * Contains information about how ingested data is processed.
     */
    readonly processingConfiguration: pulumi.Output<outputs.appfabric.IngestionDestinationProcessingConfiguration | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.appfabric.IngestionDestinationTimeouts | undefined>;
    /**
     * Create a IngestionDestination resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: IngestionDestinationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering IngestionDestination resources.
 */
export interface IngestionDestinationState {
    /**
     * The Amazon Resource Name (ARN) of the app bundle to use for the request.
     */
    appBundleArn?: pulumi.Input<string>;
    /**
     * ARN of the Ingestion Destination.
     */
    arn?: pulumi.Input<string>;
    /**
     * Contains information about the destination of ingested data.
     */
    destinationConfiguration?: pulumi.Input<inputs.appfabric.IngestionDestinationDestinationConfiguration>;
    /**
     * The Amazon Resource Name (ARN) of the ingestion to use for the request.
     */
    ingestionArn?: pulumi.Input<string>;
    /**
     * Contains information about how ingested data is processed.
     */
    processingConfiguration?: pulumi.Input<inputs.appfabric.IngestionDestinationProcessingConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.appfabric.IngestionDestinationTimeouts>;
}
/**
 * The set of arguments for constructing a IngestionDestination resource.
 */
export interface IngestionDestinationArgs {
    /**
     * The Amazon Resource Name (ARN) of the app bundle to use for the request.
     */
    appBundleArn: pulumi.Input<string>;
    /**
     * Contains information about the destination of ingested data.
     */
    destinationConfiguration?: pulumi.Input<inputs.appfabric.IngestionDestinationDestinationConfiguration>;
    /**
     * The Amazon Resource Name (ARN) of the ingestion to use for the request.
     */
    ingestionArn: pulumi.Input<string>;
    /**
     * Contains information about how ingested data is processed.
     */
    processingConfiguration?: pulumi.Input<inputs.appfabric.IngestionDestinationProcessingConfiguration>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.appfabric.IngestionDestinationTimeouts>;
}
