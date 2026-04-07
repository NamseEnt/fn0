import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS SESv2 (Simple Email V2) Dedicated IP Assignment.
 *
 * This resource is used with "Standard" dedicated IP addresses. This includes addresses [requested and relinquished manually](https://docs.aws.amazon.com/ses/latest/dg/dedicated-ip-case.html) via an AWS support case, or [Bring Your Own IP](https://docs.aws.amazon.com/ses/latest/dg/dedicated-ip-byo.html) addresses. Once no longer assigned, this resource returns the IP to the [`ses-default-dedicated-pool`](https://docs.aws.amazon.com/ses/latest/dg/managing-ip-pools.html), managed by AWS.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sesv2.DedicatedIpAssignment("example", {
 *     ip: "0.0.0.0",
 *     destinationPoolName: "my-pool",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SESv2 (Simple Email V2) Dedicated IP Assignment using the `id`, which is a comma-separated string made up of `ip` and `destination_pool_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:sesv2/dedicatedIpAssignment:DedicatedIpAssignment example "0.0.0.0,my-pool"
 * ```
 */
export declare class DedicatedIpAssignment extends pulumi.CustomResource {
    /**
     * Get an existing DedicatedIpAssignment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DedicatedIpAssignmentState, opts?: pulumi.CustomResourceOptions): DedicatedIpAssignment;
    /**
     * Returns true if the given object is an instance of DedicatedIpAssignment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DedicatedIpAssignment;
    /**
     * Dedicated IP address.
     */
    readonly destinationPoolName: pulumi.Output<string>;
    /**
     * Dedicated IP address.
     */
    readonly ip: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a DedicatedIpAssignment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DedicatedIpAssignmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DedicatedIpAssignment resources.
 */
export interface DedicatedIpAssignmentState {
    /**
     * Dedicated IP address.
     */
    destinationPoolName?: pulumi.Input<string>;
    /**
     * Dedicated IP address.
     */
    ip?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DedicatedIpAssignment resource.
 */
export interface DedicatedIpAssignmentArgs {
    /**
     * Dedicated IP address.
     */
    destinationPoolName: pulumi.Input<string>;
    /**
     * Dedicated IP address.
     */
    ip: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
