import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a CE Cost Allocation Tag.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.costexplorer.CostAllocationTag("example", {
 *     tagKey: "example",
 *     status: "Active",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_ce_cost_allocation_tag` using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:costexplorer/costAllocationTag:CostAllocationTag example key
 * ```
 */
export declare class CostAllocationTag extends pulumi.CustomResource {
    /**
     * Get an existing CostAllocationTag resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CostAllocationTagState, opts?: pulumi.CustomResourceOptions): CostAllocationTag;
    /**
     * Returns true if the given object is an instance of CostAllocationTag.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CostAllocationTag;
    /**
     * The status of a cost allocation tag. Valid values are `Active` and `Inactive`.
     */
    readonly status: pulumi.Output<string>;
    /**
     * The key for the cost allocation tag.
     */
    readonly tagKey: pulumi.Output<string>;
    /**
     * The type of cost allocation tag.
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a CostAllocationTag resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CostAllocationTagArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CostAllocationTag resources.
 */
export interface CostAllocationTagState {
    /**
     * The status of a cost allocation tag. Valid values are `Active` and `Inactive`.
     */
    status?: pulumi.Input<string>;
    /**
     * The key for the cost allocation tag.
     */
    tagKey?: pulumi.Input<string>;
    /**
     * The type of cost allocation tag.
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a CostAllocationTag resource.
 */
export interface CostAllocationTagArgs {
    /**
     * The status of a cost allocation tag. Valid values are `Active` and `Inactive`.
     */
    status: pulumi.Input<string>;
    /**
     * The key for the cost allocation tag.
     */
    tagKey: pulumi.Input<string>;
}
