import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing AWS Cost Optimization Hub Preferences.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.costoptimizationhub.Preferences("example", {});
 * ```
 *
 * ### Usage with all the arguments
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.costoptimizationhub.Preferences("example", {
 *     memberAccountDiscountVisibility: "None",
 *     savingsEstimationMode: "AfterDiscounts",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Cost Optimization Hub Preferences using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:costoptimizationhub/preferences:Preferences example 111222333444
 * ```
 */
export declare class Preferences extends pulumi.CustomResource {
    /**
     * Get an existing Preferences resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PreferencesState, opts?: pulumi.CustomResourceOptions): Preferences;
    /**
     * Returns true if the given object is an instance of Preferences.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Preferences;
    /**
     * Customize whether the member accounts can see the "After Discounts" savings estimates. Valid values are `All` and `None`. Default value is `All`.
     */
    readonly memberAccountDiscountVisibility: pulumi.Output<string>;
    /**
     * Customize how estimated monthly savings are calculated. Valid values are `BeforeDiscounts` and `AfterDiscounts`. Default value is `BeforeDiscounts`.
     */
    readonly savingsEstimationMode: pulumi.Output<string>;
    /**
     * Create a Preferences resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: PreferencesArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Preferences resources.
 */
export interface PreferencesState {
    /**
     * Customize whether the member accounts can see the "After Discounts" savings estimates. Valid values are `All` and `None`. Default value is `All`.
     */
    memberAccountDiscountVisibility?: pulumi.Input<string>;
    /**
     * Customize how estimated monthly savings are calculated. Valid values are `BeforeDiscounts` and `AfterDiscounts`. Default value is `BeforeDiscounts`.
     */
    savingsEstimationMode?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Preferences resource.
 */
export interface PreferencesArgs {
    /**
     * Customize whether the member accounts can see the "After Discounts" savings estimates. Valid values are `All` and `None`. Default value is `All`.
     */
    memberAccountDiscountVisibility?: pulumi.Input<string>;
    /**
     * Customize how estimated monthly savings are calculated. Valid values are `BeforeDiscounts` and `AfterDiscounts`. Default value is `BeforeDiscounts`.
     */
    savingsEstimationMode?: pulumi.Input<string>;
}
