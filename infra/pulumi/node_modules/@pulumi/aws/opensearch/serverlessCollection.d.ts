import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS OpenSearch Serverless Collection.
 *
 * > **NOTE:** An `aws.opensearch.ServerlessCollection` cannot be created without having an applicable encryption security policy. Use the `dependsOn` meta-argument to define this dependency.
 *
 * > **NOTE:** An `aws.opensearch.ServerlessCollection` is not accessible without configuring an applicable network security policy. Data cannot be accessed without configuring an applicable data access policy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.opensearch.ServerlessSecurityPolicy("example", {
 *     name: "example",
 *     type: "encryption",
 *     policy: JSON.stringify({
 *         Rules: [{
 *             Resource: ["collection/example"],
 *             ResourceType: "collection",
 *         }],
 *         AWSOwnedKey: true,
 *     }),
 * });
 * const exampleServerlessCollection = new aws.opensearch.ServerlessCollection("example", {name: "example"}, {
 *     dependsOn: [example],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import OpenSearchServerless Collection using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:opensearch/serverlessCollection:ServerlessCollection example example
 * ```
 */
export declare class ServerlessCollection extends pulumi.CustomResource {
    /**
     * Get an existing ServerlessCollection resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ServerlessCollectionState, opts?: pulumi.CustomResourceOptions): ServerlessCollection;
    /**
     * Returns true if the given object is an instance of ServerlessCollection.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ServerlessCollection;
    /**
     * Amazon Resource Name (ARN) of the collection.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Collection-specific endpoint used to submit index, search, and data upload requests to an OpenSearch Serverless collection.
     */
    readonly collectionEndpoint: pulumi.Output<string>;
    /**
     * Collection-specific endpoint used to access OpenSearch Dashboards.
     */
    readonly dashboardEndpoint: pulumi.Output<string>;
    /**
     * Description of the collection.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The ARN of the Amazon Web Services KMS key used to encrypt the collection.
     */
    readonly kmsKeyArn: pulumi.Output<string>;
    /**
     * Name of the collection.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Indicates whether standby replicas should be used for a collection. One of `ENABLED` or `DISABLED`. Defaults to `ENABLED`.
     */
    readonly standbyReplicas: pulumi.Output<string>;
    /**
     * A map of tags to assign to the collection. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.opensearch.ServerlessCollectionTimeouts | undefined>;
    /**
     * Type of collection. One of `SEARCH`, `TIMESERIES`, or `VECTORSEARCH`. Defaults to `TIMESERIES`.
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a ServerlessCollection resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ServerlessCollectionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ServerlessCollection resources.
 */
export interface ServerlessCollectionState {
    /**
     * Amazon Resource Name (ARN) of the collection.
     */
    arn?: pulumi.Input<string>;
    /**
     * Collection-specific endpoint used to submit index, search, and data upload requests to an OpenSearch Serverless collection.
     */
    collectionEndpoint?: pulumi.Input<string>;
    /**
     * Collection-specific endpoint used to access OpenSearch Dashboards.
     */
    dashboardEndpoint?: pulumi.Input<string>;
    /**
     * Description of the collection.
     */
    description?: pulumi.Input<string>;
    /**
     * The ARN of the Amazon Web Services KMS key used to encrypt the collection.
     */
    kmsKeyArn?: pulumi.Input<string>;
    /**
     * Name of the collection.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Indicates whether standby replicas should be used for a collection. One of `ENABLED` or `DISABLED`. Defaults to `ENABLED`.
     */
    standbyReplicas?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the collection. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.opensearch.ServerlessCollectionTimeouts>;
    /**
     * Type of collection. One of `SEARCH`, `TIMESERIES`, or `VECTORSEARCH`. Defaults to `TIMESERIES`.
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ServerlessCollection resource.
 */
export interface ServerlessCollectionArgs {
    /**
     * Description of the collection.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the collection.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Indicates whether standby replicas should be used for a collection. One of `ENABLED` or `DISABLED`. Defaults to `ENABLED`.
     */
    standbyReplicas?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the collection. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.opensearch.ServerlessCollectionTimeouts>;
    /**
     * Type of collection. One of `SEARCH`, `TIMESERIES`, or `VECTORSEARCH`. Defaults to `TIMESERIES`.
     */
    type?: pulumi.Input<string>;
}
