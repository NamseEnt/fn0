import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS Route 53 Recovery Control Config Cluster.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.route53recoverycontrol.Cluster("example", {name: "georgefitzgerald"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Route53 Recovery Control Config cluster using the cluster ARN. For example:
 *
 * ```sh
 * $ pulumi import aws:route53recoverycontrol/cluster:Cluster mycluster arn:aws:route53-recovery-control::313517334327:cluster/f9ae13be-a11e-4ec7-8522-94a70468e6ea
 * ```
 */
export declare class Cluster extends pulumi.CustomResource {
    /**
     * Get an existing Cluster resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClusterState, opts?: pulumi.CustomResourceOptions): Cluster;
    /**
     * Returns true if the given object is an instance of Cluster.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Cluster;
    /**
     * ARN of the cluster
     */
    readonly arn: pulumi.Output<string>;
    /**
     * List of 5 endpoints in 5 regions that can be used to talk to the cluster. See below.
     */
    readonly clusterEndpoints: pulumi.Output<outputs.route53recoverycontrol.ClusterClusterEndpoint[]>;
    /**
     * Unique name describing the cluster.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Network type of cluster. Valid values are `IPV4` and `DUALSTACK`. Defaults to `IPV4`.
     *
     * The following arguments are optional:
     */
    readonly networkType: pulumi.Output<string>;
    /**
     * Status of cluster. `PENDING` when it is being created, `PENDING_DELETION` when it is being deleted and `DEPLOYED` otherwise.
     */
    readonly status: pulumi.Output<string>;
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
     * Create a Cluster resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ClusterArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Cluster resources.
 */
export interface ClusterState {
    /**
     * ARN of the cluster
     */
    arn?: pulumi.Input<string>;
    /**
     * List of 5 endpoints in 5 regions that can be used to talk to the cluster. See below.
     */
    clusterEndpoints?: pulumi.Input<pulumi.Input<inputs.route53recoverycontrol.ClusterClusterEndpoint>[]>;
    /**
     * Unique name describing the cluster.
     */
    name?: pulumi.Input<string>;
    /**
     * Network type of cluster. Valid values are `IPV4` and `DUALSTACK`. Defaults to `IPV4`.
     *
     * The following arguments are optional:
     */
    networkType?: pulumi.Input<string>;
    /**
     * Status of cluster. `PENDING` when it is being created, `PENDING_DELETION` when it is being deleted and `DEPLOYED` otherwise.
     */
    status?: pulumi.Input<string>;
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
 * The set of arguments for constructing a Cluster resource.
 */
export interface ClusterArgs {
    /**
     * Unique name describing the cluster.
     */
    name?: pulumi.Input<string>;
    /**
     * Network type of cluster. Valid values are `IPV4` and `DUALSTACK`. Defaults to `IPV4`.
     *
     * The following arguments are optional:
     */
    networkType?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
