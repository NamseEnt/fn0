import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a WAF Regional Size Constraint Set Resource for use with Application Load Balancer.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const sizeConstraintSet = new aws.wafregional.SizeConstraintSet("size_constraint_set", {
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
 * Using `pulumi import`, import WAF Size Constraint Set using the id. For example:
 *
 * ```sh
 * $ pulumi import aws:wafregional/sizeConstraintSet:SizeConstraintSet size_constraint_set a1b2c3d4-d5f6-7777-8888-9999aaaabbbbcccc
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
    readonly arn: pulumi.Output<string>;
    /**
     * The name or description of the Size Constraint Set.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Specifies the parts of web requests that you want to inspect the size of.
     */
    readonly sizeConstraints: pulumi.Output<outputs.wafregional.SizeConstraintSetSizeConstraint[] | undefined>;
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
    arn?: pulumi.Input<string>;
    /**
     * The name or description of the Size Constraint Set.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the parts of web requests that you want to inspect the size of.
     */
    sizeConstraints?: pulumi.Input<pulumi.Input<inputs.wafregional.SizeConstraintSetSizeConstraint>[]>;
}
/**
 * The set of arguments for constructing a SizeConstraintSet resource.
 */
export interface SizeConstraintSetArgs {
    /**
     * The name or description of the Size Constraint Set.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Specifies the parts of web requests that you want to inspect the size of.
     */
    sizeConstraints?: pulumi.Input<pulumi.Input<inputs.wafregional.SizeConstraintSetSizeConstraint>[]>;
}
