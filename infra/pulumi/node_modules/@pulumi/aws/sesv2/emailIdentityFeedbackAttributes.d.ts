import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS SESv2 (Simple Email V2) Email Identity Feedback Attributes.
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
 * const exampleEmailIdentityFeedbackAttributes = new aws.sesv2.EmailIdentityFeedbackAttributes("example", {
 *     emailIdentity: example.emailIdentity,
 *     emailForwardingEnabled: true,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SESv2 (Simple Email V2) Email Identity Feedback Attributes using the `email_identity`. For example:
 *
 * ```sh
 * $ pulumi import aws:sesv2/emailIdentityFeedbackAttributes:EmailIdentityFeedbackAttributes example example.com
 * ```
 */
export declare class EmailIdentityFeedbackAttributes extends pulumi.CustomResource {
    /**
     * Get an existing EmailIdentityFeedbackAttributes resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EmailIdentityFeedbackAttributesState, opts?: pulumi.CustomResourceOptions): EmailIdentityFeedbackAttributes;
    /**
     * Returns true if the given object is an instance of EmailIdentityFeedbackAttributes.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EmailIdentityFeedbackAttributes;
    /**
     * Sets the feedback forwarding configuration for the identity.
     */
    readonly emailForwardingEnabled: pulumi.Output<boolean | undefined>;
    /**
     * The email identity.
     */
    readonly emailIdentity: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a EmailIdentityFeedbackAttributes resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EmailIdentityFeedbackAttributesArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EmailIdentityFeedbackAttributes resources.
 */
export interface EmailIdentityFeedbackAttributesState {
    /**
     * Sets the feedback forwarding configuration for the identity.
     */
    emailForwardingEnabled?: pulumi.Input<boolean>;
    /**
     * The email identity.
     */
    emailIdentity?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EmailIdentityFeedbackAttributes resource.
 */
export interface EmailIdentityFeedbackAttributesArgs {
    /**
     * Sets the feedback forwarding configuration for the identity.
     */
    emailForwardingEnabled?: pulumi.Input<boolean>;
    /**
     * The email identity.
     */
    emailIdentity: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
