import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Neptune Cluster Endpoint Resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.neptune.ClusterEndpoint("example", {
 *     clusterIdentifier: test.clusterIdentifier,
 *     clusterEndpointIdentifier: "example",
 *     endpointType: "READER",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_neptune_cluster_endpoint` using the `cluster-identifier:endpoint-identfier`. For example:
 *
 * ```sh
 * $ pulumi import aws:neptune/clusterEndpoint:ClusterEndpoint example my-cluster:my-endpoint
 * ```
 */
export declare class ClusterEndpoint extends pulumi.CustomResource {
    /**
     * Get an existing ClusterEndpoint resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClusterEndpointState, opts?: pulumi.CustomResourceOptions): ClusterEndpoint;
    /**
     * Returns true if the given object is an instance of ClusterEndpoint.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ClusterEndpoint;
    /**
     * The Neptune Cluster Endpoint Amazon Resource Name (ARN).
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The identifier of the endpoint.
     */
    readonly clusterEndpointIdentifier: pulumi.Output<string>;
    /**
     * The DB cluster identifier of the DB cluster associated with the endpoint.
     */
    readonly clusterIdentifier: pulumi.Output<string>;
    /**
     * The DNS address of the endpoint.
     */
    readonly endpoint: pulumi.Output<string>;
    /**
     * The type of the endpoint. One of: `READER`, `WRITER`, `ANY`.
     */
    readonly endpointType: pulumi.Output<string>;
    /**
     * List of DB instance identifiers that aren't part of the custom endpoint group. All other eligible instances are reachable through the custom endpoint. Only relevant if the list of static members is empty.
     */
    readonly excludedMembers: pulumi.Output<string[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * List of DB instance identifiers that are part of the custom endpoint group.
     */
    readonly staticMembers: pulumi.Output<string[] | undefined>;
    /**
     * A map of tags to assign to the Neptune cluster. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a ClusterEndpoint resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ClusterEndpointArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ClusterEndpoint resources.
 */
export interface ClusterEndpointState {
    /**
     * The Neptune Cluster Endpoint Amazon Resource Name (ARN).
     */
    arn?: pulumi.Input<string>;
    /**
     * The identifier of the endpoint.
     */
    clusterEndpointIdentifier?: pulumi.Input<string>;
    /**
     * The DB cluster identifier of the DB cluster associated with the endpoint.
     */
    clusterIdentifier?: pulumi.Input<string>;
    /**
     * The DNS address of the endpoint.
     */
    endpoint?: pulumi.Input<string>;
    /**
     * The type of the endpoint. One of: `READER`, `WRITER`, `ANY`.
     */
    endpointType?: pulumi.Input<string>;
    /**
     * List of DB instance identifiers that aren't part of the custom endpoint group. All other eligible instances are reachable through the custom endpoint. Only relevant if the list of static members is empty.
     */
    excludedMembers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of DB instance identifiers that are part of the custom endpoint group.
     */
    staticMembers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A map of tags to assign to the Neptune cluster. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a ClusterEndpoint resource.
 */
export interface ClusterEndpointArgs {
    /**
     * The identifier of the endpoint.
     */
    clusterEndpointIdentifier: pulumi.Input<string>;
    /**
     * The DB cluster identifier of the DB cluster associated with the endpoint.
     */
    clusterIdentifier: pulumi.Input<string>;
    /**
     * The type of the endpoint. One of: `READER`, `WRITER`, `ANY`.
     */
    endpointType: pulumi.Input<string>;
    /**
     * List of DB instance identifiers that aren't part of the custom endpoint group. All other eligible instances are reachable through the custom endpoint. Only relevant if the list of static members is empty.
     */
    excludedMembers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of DB instance identifiers that are part of the custom endpoint group.
     */
    staticMembers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A map of tags to assign to the Neptune cluster. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
