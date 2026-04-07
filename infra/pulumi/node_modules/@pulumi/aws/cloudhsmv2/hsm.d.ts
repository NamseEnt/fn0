import * as pulumi from "@pulumi/pulumi";
/**
 * Creates an HSM module in Amazon CloudHSM v2 cluster.
 *
 * ## Example Usage
 *
 * The following example below creates an HSM module in CloudHSM cluster.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const cluster = aws.cloudhsmv2.getCluster({
 *     clusterId: cloudhsmClusterId,
 * });
 * const cloudhsmV2Hsm = new aws.cloudhsmv2.Hsm("cloudhsm_v2_hsm", {
 *     subnetId: cluster.then(cluster => cluster.subnetIds?.[0]),
 *     clusterId: cluster.then(cluster => cluster.clusterId),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import HSM modules using their HSM ID. For example:
 *
 * ```sh
 * $ pulumi import aws:cloudhsmv2/hsm:Hsm bar hsm-quo8dahtaca
 * ```
 */
export declare class Hsm extends pulumi.CustomResource {
    /**
     * Get an existing Hsm resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: HsmState, opts?: pulumi.CustomResourceOptions): Hsm;
    /**
     * Returns true if the given object is an instance of Hsm.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Hsm;
    /**
     * The IDs of AZ in which HSM module will be located. Conflicts with `subnetId`.
     */
    readonly availabilityZone: pulumi.Output<string>;
    /**
     * The ID of Cloud HSM v2 cluster to which HSM will be added.
     */
    readonly clusterId: pulumi.Output<string>;
    /**
     * The id of the ENI interface allocated for HSM module.
     */
    readonly hsmEniId: pulumi.Output<string>;
    /**
     * The id of the HSM module.
     */
    readonly hsmId: pulumi.Output<string>;
    /**
     * The state of the HSM module.
     */
    readonly hsmState: pulumi.Output<string>;
    /**
     * The IP address of HSM module. Must be within the CIDR of selected subnet.
     *
     * > **NOTE:** Either `subnetId` or `availabilityZone` must be specified.
     */
    readonly ipAddress: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of subnet in which HSM module will be located. Conflicts with `availabilityZone`.
     */
    readonly subnetId: pulumi.Output<string>;
    /**
     * Create a Hsm resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: HsmArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Hsm resources.
 */
export interface HsmState {
    /**
     * The IDs of AZ in which HSM module will be located. Conflicts with `subnetId`.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * The ID of Cloud HSM v2 cluster to which HSM will be added.
     */
    clusterId?: pulumi.Input<string>;
    /**
     * The id of the ENI interface allocated for HSM module.
     */
    hsmEniId?: pulumi.Input<string>;
    /**
     * The id of the HSM module.
     */
    hsmId?: pulumi.Input<string>;
    /**
     * The state of the HSM module.
     */
    hsmState?: pulumi.Input<string>;
    /**
     * The IP address of HSM module. Must be within the CIDR of selected subnet.
     *
     * > **NOTE:** Either `subnetId` or `availabilityZone` must be specified.
     */
    ipAddress?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of subnet in which HSM module will be located. Conflicts with `availabilityZone`.
     */
    subnetId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Hsm resource.
 */
export interface HsmArgs {
    /**
     * The IDs of AZ in which HSM module will be located. Conflicts with `subnetId`.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * The ID of Cloud HSM v2 cluster to which HSM will be added.
     */
    clusterId: pulumi.Input<string>;
    /**
     * The IP address of HSM module. Must be within the CIDR of selected subnet.
     *
     * > **NOTE:** Either `subnetId` or `availabilityZone` must be specified.
     */
    ipAddress?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of subnet in which HSM module will be located. Conflicts with `availabilityZone`.
     */
    subnetId?: pulumi.Input<string>;
}
