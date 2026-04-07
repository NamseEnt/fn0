import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * The `aws.neptunegraph.Graph` resource creates an Amazon Analytics Graph.
 *
 * ## Example Usage
 *
 * ### Neptune Graph (with Vector Search configuration)
 *
 * Creates a Neptune Graph with 16GB provisioned memory, vector search capability with 128 dimensions, and a single replica for high availability.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * // Create Neptune Graph
 * const example = new aws.neptunegraph.Graph("example", {
 *     graphName: "example-graph-test-20250203",
 *     provisionedMemory: 16,
 *     deletionProtection: false,
 *     publicConnectivity: false,
 *     replicaCount: 1,
 *     kmsKeyIdentifier: "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789012",
 *     vectorSearchConfiguration: {
 *         vectorSearchDimension: 128,
 *     },
 *     tags: {
 *         Environment: "Development",
 *         ModifiedBy: "AWS",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_neptunegraph_graph` using the graph identifier. For example:
 *
 * ```sh
 * $ pulumi import aws:neptunegraph/graph:Graph example "graph_id"
 * ```
 */
export declare class Graph extends pulumi.CustomResource {
    /**
     * Get an existing Graph resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GraphState, opts?: pulumi.CustomResourceOptions): Graph;
    /**
     * Returns true if the given object is an instance of Graph.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Graph;
    /**
     * (String) Graph resource ARN
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Value that indicates whether the Graph has deletion protection enabled. The graph can't be deleted when deletion protection is enabled.
     */
    readonly deletionProtection: pulumi.Output<boolean>;
    /**
     * (String) The connection endpoint for the graph. For example: `g-12a3bcdef4.us-east-1.neptune-graph.amazonaws.com`
     */
    readonly endpoint: pulumi.Output<string>;
    /**
     * The graph name. For example: my-graph-1.
     * 								The name must contain from 1 to 63 letters, numbers, or hyphens,
     * 								and its first character must be a letter. It cannot end with a hyphen or contain two consecutive hyphens.
     * 								If you don't specify a graph name, a unique graph name is generated for you using the prefix graph-for,
     * 								followed by a combination of Stack Name and a UUID.
     */
    readonly graphName: pulumi.Output<string>;
    /**
     * Allows user to specify name prefix and have remainder of name automatically generated.
     */
    readonly graphNamePrefix: pulumi.Output<string | undefined>;
    /**
     * The ARN for the KMS encryption key. By Default, Neptune Analytics will use an AWS provided key ("AWS_OWNED_KEY"). This parameter is used if you want to encrypt the graph using a KMS Customer Managed Key (CMK).
     */
    readonly kmsKeyIdentifier: pulumi.Output<string>;
    /**
     * The provisioned memory-optimized Neptune Capacity Units (m-NCUs) to use for the graph.
     *
     * The following arguments are optional:
     */
    readonly provisionedMemory: pulumi.Output<number>;
    /**
     * Specifies whether the Graph can be reached over the internet. Access to all graphs requires IAM authentication.  When the Graph is publicly reachable, its Domain Name System (DNS) endpoint resolves to the public IP address from the internet.  When the Graph isn't publicly reachable, you need to create a PrivateGraphEndpoint in a given VPC to ensure the DNS name resolves to a private IP address that is reachable from the VPC.
     */
    readonly publicConnectivity: pulumi.Output<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specifies the number of replicas you want when finished. All replicas will be provisioned in different availability zones.  Replica Count should always be less than or equal to 2.
     */
    readonly replicaCount: pulumi.Output<number>;
    /**
     * Key-value tags for the graph. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly timeouts: pulumi.Output<outputs.neptunegraph.GraphTimeouts | undefined>;
    /**
     * Vector Search Configuration (see below for nested schema of vector_search_configuration)
     */
    readonly vectorSearchConfiguration: pulumi.Output<outputs.neptunegraph.GraphVectorSearchConfiguration | undefined>;
    /**
     * Create a Graph resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: GraphArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Graph resources.
 */
export interface GraphState {
    /**
     * (String) Graph resource ARN
     */
    arn?: pulumi.Input<string>;
    /**
     * Value that indicates whether the Graph has deletion protection enabled. The graph can't be deleted when deletion protection is enabled.
     */
    deletionProtection?: pulumi.Input<boolean>;
    /**
     * (String) The connection endpoint for the graph. For example: `g-12a3bcdef4.us-east-1.neptune-graph.amazonaws.com`
     */
    endpoint?: pulumi.Input<string>;
    /**
     * The graph name. For example: my-graph-1.
     * 								The name must contain from 1 to 63 letters, numbers, or hyphens,
     * 								and its first character must be a letter. It cannot end with a hyphen or contain two consecutive hyphens.
     * 								If you don't specify a graph name, a unique graph name is generated for you using the prefix graph-for,
     * 								followed by a combination of Stack Name and a UUID.
     */
    graphName?: pulumi.Input<string>;
    /**
     * Allows user to specify name prefix and have remainder of name automatically generated.
     */
    graphNamePrefix?: pulumi.Input<string>;
    /**
     * The ARN for the KMS encryption key. By Default, Neptune Analytics will use an AWS provided key ("AWS_OWNED_KEY"). This parameter is used if you want to encrypt the graph using a KMS Customer Managed Key (CMK).
     */
    kmsKeyIdentifier?: pulumi.Input<string>;
    /**
     * The provisioned memory-optimized Neptune Capacity Units (m-NCUs) to use for the graph.
     *
     * The following arguments are optional:
     */
    provisionedMemory?: pulumi.Input<number>;
    /**
     * Specifies whether the Graph can be reached over the internet. Access to all graphs requires IAM authentication.  When the Graph is publicly reachable, its Domain Name System (DNS) endpoint resolves to the public IP address from the internet.  When the Graph isn't publicly reachable, you need to create a PrivateGraphEndpoint in a given VPC to ensure the DNS name resolves to a private IP address that is reachable from the VPC.
     */
    publicConnectivity?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the number of replicas you want when finished. All replicas will be provisioned in different availability zones.  Replica Count should always be less than or equal to 2.
     */
    replicaCount?: pulumi.Input<number>;
    /**
     * Key-value tags for the graph. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    timeouts?: pulumi.Input<inputs.neptunegraph.GraphTimeouts>;
    /**
     * Vector Search Configuration (see below for nested schema of vector_search_configuration)
     */
    vectorSearchConfiguration?: pulumi.Input<inputs.neptunegraph.GraphVectorSearchConfiguration>;
}
/**
 * The set of arguments for constructing a Graph resource.
 */
export interface GraphArgs {
    /**
     * Value that indicates whether the Graph has deletion protection enabled. The graph can't be deleted when deletion protection is enabled.
     */
    deletionProtection?: pulumi.Input<boolean>;
    /**
     * The graph name. For example: my-graph-1.
     * 								The name must contain from 1 to 63 letters, numbers, or hyphens,
     * 								and its first character must be a letter. It cannot end with a hyphen or contain two consecutive hyphens.
     * 								If you don't specify a graph name, a unique graph name is generated for you using the prefix graph-for,
     * 								followed by a combination of Stack Name and a UUID.
     */
    graphName?: pulumi.Input<string>;
    /**
     * Allows user to specify name prefix and have remainder of name automatically generated.
     */
    graphNamePrefix?: pulumi.Input<string>;
    /**
     * The ARN for the KMS encryption key. By Default, Neptune Analytics will use an AWS provided key ("AWS_OWNED_KEY"). This parameter is used if you want to encrypt the graph using a KMS Customer Managed Key (CMK).
     */
    kmsKeyIdentifier?: pulumi.Input<string>;
    /**
     * The provisioned memory-optimized Neptune Capacity Units (m-NCUs) to use for the graph.
     *
     * The following arguments are optional:
     */
    provisionedMemory: pulumi.Input<number>;
    /**
     * Specifies whether the Graph can be reached over the internet. Access to all graphs requires IAM authentication.  When the Graph is publicly reachable, its Domain Name System (DNS) endpoint resolves to the public IP address from the internet.  When the Graph isn't publicly reachable, you need to create a PrivateGraphEndpoint in a given VPC to ensure the DNS name resolves to a private IP address that is reachable from the VPC.
     */
    publicConnectivity?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the number of replicas you want when finished. All replicas will be provisioned in different availability zones.  Replica Count should always be less than or equal to 2.
     */
    replicaCount?: pulumi.Input<number>;
    /**
     * Key-value tags for the graph. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.neptunegraph.GraphTimeouts>;
    /**
     * Vector Search Configuration (see below for nested schema of vector_search_configuration)
     */
    vectorSearchConfiguration?: pulumi.Input<inputs.neptunegraph.GraphVectorSearchConfiguration>;
}
