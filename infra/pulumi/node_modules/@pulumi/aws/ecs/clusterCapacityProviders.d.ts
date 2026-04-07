import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages the capacity providers of an ECS Cluster.
 *
 * More information about capacity providers can be found in the [ECS User Guide](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/cluster-capacity-providers.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ecs.Cluster("example", {name: "my-cluster"});
 * const exampleClusterCapacityProviders = new aws.ecs.ClusterCapacityProviders("example", {
 *     clusterName: example.name,
 *     capacityProviders: ["FARGATE"],
 *     defaultCapacityProviderStrategies: [{
 *         base: 1,
 *         weight: 100,
 *         capacityProvider: "FARGATE",
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import ECS cluster capacity providers using the `cluster_name` attribute. For example:
 *
 * ```sh
 * $ pulumi import aws:ecs/clusterCapacityProviders:ClusterCapacityProviders example my-cluster
 * ```
 */
export declare class ClusterCapacityProviders extends pulumi.CustomResource {
    /**
     * Get an existing ClusterCapacityProviders resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClusterCapacityProvidersState, opts?: pulumi.CustomResourceOptions): ClusterCapacityProviders;
    /**
     * Returns true if the given object is an instance of ClusterCapacityProviders.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ClusterCapacityProviders;
    /**
     * Set of names of one or more capacity providers to associate with the cluster. Valid values also include `FARGATE` and `FARGATE_SPOT`.
     */
    readonly capacityProviders: pulumi.Output<string[] | undefined>;
    /**
     * Name of the ECS cluster to manage capacity providers for.
     */
    readonly clusterName: pulumi.Output<string>;
    /**
     * Set of capacity provider strategies to use by default for the cluster. Detailed below.
     */
    readonly defaultCapacityProviderStrategies: pulumi.Output<outputs.ecs.ClusterCapacityProvidersDefaultCapacityProviderStrategy[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a ClusterCapacityProviders resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ClusterCapacityProvidersArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ClusterCapacityProviders resources.
 */
export interface ClusterCapacityProvidersState {
    /**
     * Set of names of one or more capacity providers to associate with the cluster. Valid values also include `FARGATE` and `FARGATE_SPOT`.
     */
    capacityProviders?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Name of the ECS cluster to manage capacity providers for.
     */
    clusterName?: pulumi.Input<string>;
    /**
     * Set of capacity provider strategies to use by default for the cluster. Detailed below.
     */
    defaultCapacityProviderStrategies?: pulumi.Input<pulumi.Input<inputs.ecs.ClusterCapacityProvidersDefaultCapacityProviderStrategy>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ClusterCapacityProviders resource.
 */
export interface ClusterCapacityProvidersArgs {
    /**
     * Set of names of one or more capacity providers to associate with the cluster. Valid values also include `FARGATE` and `FARGATE_SPOT`.
     */
    capacityProviders?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Name of the ECS cluster to manage capacity providers for.
     */
    clusterName: pulumi.Input<string>;
    /**
     * Set of capacity provider strategies to use by default for the cluster. Detailed below.
     */
    defaultCapacityProviderStrategies?: pulumi.Input<pulumi.Input<inputs.ecs.ClusterCapacityProvidersDefaultCapacityProviderStrategy>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
