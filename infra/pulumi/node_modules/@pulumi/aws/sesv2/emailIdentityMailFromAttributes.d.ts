import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS SESv2 (Simple Email V2) Email Identity Mail From Attributes.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sesv2.EmailIdentity("example", {emailIdentity: "example.com"});
 * const exampleEmailIdentityMailFromAttributes = new aws.sesv2.EmailIdentityMailFromAttributes("example", {
 *     emailIdentity: example.emailIdentity,
 *     behaviorOnMxFailure: "REJECT_MESSAGE",
 *     mailFromDomain: pulumi.interpolate`subdomain.${example.emailIdentity}`,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SESv2 (Simple Email V2) Email Identity Mail From Attributes using the `email_identity`. For example:
 *
 * ```sh
 * $ pulumi import aws:sesv2/emailIdentityMailFromAttributes:EmailIdentityMailFromAttributes example example.com
 * ```
 */
export declare class EmailIdentityMailFromAttributes extends pulumi.CustomResource {
    /**
     * Get an existing EmailIdentityMailFromAttributes resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EmailIdentityMailFromAttributesState, opts?: pulumi.CustomResourceOptions): EmailIdentityMailFromAttributes;
    /**
     * Returns true if the given object is an instance of EmailIdentityMailFromAttributes.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EmailIdentityMailFromAttributes;
    /**
     * The action to take if the required MX record isn't found when you send an email. Valid values: `USE_DEFAULT_VALUE`, `REJECT_MESSAGE`.
     */
    readonly behaviorOnMxFailure: pulumi.Output<string | undefined>;
    /**
     * The verified email identity.
     */
    readonly emailIdentity: pulumi.Output<string>;
    /**
     * The custom MAIL FROM domain that you want the verified identity to use. Required if `behaviorOnMxFailure` is `REJECT_MESSAGE`.
     */
    readonly mailFromDomain: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a EmailIdentityMailFromAttributes resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EmailIdentityMailFromAttributesArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EmailIdentityMailFromAttributes resources.
 */
export interface EmailIdentityMailFromAttributesState {
    /**
     * The action to take if the required MX record isn't found when you send an email. Valid values: `USE_DEFAULT_VALUE`, `REJECT_MESSAGE`.
     */
    behaviorOnMxFailure?: pulumi.Input<string>;
    /**
     * The verified email identity.
     */
    emailIdentity?: pulumi.Input<string>;
    /**
     * The custom MAIL FROM domain that you want the verified identity to use. Required if `behaviorOnMxFailure` is `REJECT_MESSAGE`.
     */
    mailFromDomain?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EmailIdentityMailFromAttributes resource.
 */
export interface EmailIdentityMailFromAttributesArgs {
    /**
     * The action to take if the required MX record isn't found when you send an email. Valid values: `USE_DEFAULT_VALUE`, `REJECT_MESSAGE`.
     */
    behaviorOnMxFailure?: pulumi.Input<string>;
    /**
     * The verified email identity.
     */
    emailIdentity: pulumi.Input<string>;
    /**
     * The custom MAIL FROM domain that you want the verified identity to use. Required if `behaviorOnMxFailure` is `REJECT_MESSAGE`.
     */
    mailFromDomain?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
