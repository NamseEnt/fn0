import * as pulumi from "@pulumi/pulumi";
/**
 * Manages AWS SESv2 (Simple Email V2) account-level suppression attributes.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sesv2.AccountSuppressionAttributes("example", {suppressedReasons: ["COMPLAINT"]});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import account-level suppression attributes using the account ID. For example:
 *
 * ```sh
 * $ pulumi import aws:sesv2/accountSuppressionAttributes:AccountSuppressionAttributes example 123456789012
 * ```
 */
export declare class AccountSuppressionAttributes extends pulumi.CustomResource {
    /**
     * Get an existing AccountSuppressionAttributes resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AccountSuppressionAttributesState, opts?: pulumi.CustomResourceOptions): AccountSuppressionAttributes;
    /**
     * Returns true if the given object is an instance of AccountSuppressionAttributes.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AccountSuppressionAttributes;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A list that contains the reasons that email addresses will be automatically added to the suppression list for your account. Valid values: `COMPLAINT`, `BOUNCE`.
     */
    readonly suppressedReasons: pulumi.Output<string[]>;
    /**
     * Create a AccountSuppressionAttributes resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AccountSuppressionAttributesArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AccountSuppressionAttributes resources.
 */
export interface AccountSuppressionAttributesState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list that contains the reasons that email addresses will be automatically added to the suppression list for your account. Valid values: `COMPLAINT`, `BOUNCE`.
     */
    suppressedReasons?: pulumi.Input<pulumi.Input<string>[]>;
}
/**
 * The set of arguments for constructing a AccountSuppressionAttributes resource.
 */
export interface AccountSuppressionAttributesArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list that contains the reasons that email addresses will be automatically added to the suppression list for your account. Valid values: `COMPLAINT`, `BOUNCE`.
     */
    suppressedReasons: pulumi.Input<pulumi.Input<string>[]>;
}
