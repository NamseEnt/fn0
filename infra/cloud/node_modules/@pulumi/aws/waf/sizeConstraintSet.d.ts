import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Use the `aws.waf.SizeConstraintSet` resource to manage WAF size constraint sets.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const sizeConstraintSet = new aws.waf.SizeConstraintSet("size_constraint_set", {
 *     name: "tfsize_constraints",
 *     sizeConstraints: [{
 *         textTransformation: "NONE",
 *         comparisonOperator: "EQ",
 *         size: 4096,
 *         fieldToMatch: {
 *             type: "BODY",
 *         },
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AWS WAF Size Constraint Set using their ID. For example:
 *
 * ```sh
 * $ pulumi import aws:waf/sizeConstraintSet:SizeConstraintSet example a1b2c3d4-d5f6-7777-8888-9999aaaabbbbcccc
 * ```
 */
export declare class SizeConstraintSet extends pulumi.CustomResource {
    /**
     * Get an existing SizeConstraintSet resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SizeConstraintSetState, opts?: pulumi.CustomResourceOptions): SizeConstraintSet;
    /**
     * Returns true if the given object is an instance of SizeConstraintSet.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SizeConstraintSet;
    /**
     * Amazon Resource Name (ARN).
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Name or description of the Size Constraint Set.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Parts of web requests that you want to inspect the size of.
     */
    readonly sizeConstraints: pulumi.Output<outputs.waf.SizeConstraintSetSizeConstraint[] | undefined>;
    /**
     * Create a SizeConstraintSet resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: SizeConstraintSetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SizeConstraintSet resources.
 */
export interface SizeConstraintSetState {
    /**
     * Amazon Resource Name (ARN).
     */
    arn?: pulumi.Input<string>;
    /**
     * Name or description of the Size Constraint Set.
     */
    name?: pulumi.Input<string>;
    /**
     * Parts of web requests that you want to inspect the size of.
     */
    sizeConstraints?: pulumi.Input<pulumi.Input<inputs.waf.SizeConstraintSetSizeConstraint>[]>;
}
/**
 * The set of arguments for constructing a SizeConstraintSet resource.
 */
export interface SizeConstraintSetArgs {
    /**
     * Name or description of the Size Constraint Set.
     */
    name?: pulumi.Input<string>;
    /**
     * Parts of web requests that you want to inspect the size of.
     */
    sizeConstraints?: pulumi.Input<pulumi.Input<inputs.waf.SizeConstraintSetSizeConstraint>[]>;
}
