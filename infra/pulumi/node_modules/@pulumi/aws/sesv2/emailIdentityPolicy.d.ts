import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS SESv2 (Simple Email V2) Email Identity Policy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sesv2.EmailIdentity("example", {emailIdentity: "testing@example.com"});
 * const exampleEmailIdentityPolicy = new aws.sesv2.EmailIdentityPolicy("example", {
 *     emailIdentity: example.emailIdentity,
 *     policyName: "example",
 *     policy: pulumi.interpolate`{
 *   \"Id\":\"ExampleAuthorizationPolicy\",
 *   \"Version\":\"2012-10-17\",
 *   \"Statement\":[
 *     {
 *       \"Sid\":\"AuthorizeIAMUser\",
 *       \"Effect\":\"Allow\",
 *       \"Resource\":\"${example.arn}\",
 *       \"Principal\":{
 *         \"AWS\":[
 *           \"arn:aws:iam::123456789012:user/John\",
 *           \"arn:aws:iam::123456789012:user/Jane\"
 *         ]
 *       },
 *       \"Action\":[
 *         \"ses:DeleteEmailIdentity\",
 *         \"ses:PutEmailIdentityDkimSigningAttributes\"
 *       ]
 *     }
 *   ]
 * }
 * `,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SESv2 (Simple Email V2) Email Identity Policy using the `email_identity` and `policy_name` separated by `|`. For example:
 *
 * ```sh
 * $ pulumi import aws:sesv2/emailIdentityPolicy:EmailIdentityPolicy example example_email_identity|example_policy_name
 * ```
 */
export declare class EmailIdentityPolicy extends pulumi.CustomResource {
    /**
     * Get an existing EmailIdentityPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EmailIdentityPolicyState, opts?: pulumi.CustomResourceOptions): EmailIdentityPolicy;
    /**
     * Returns true if the given object is an instance of EmailIdentityPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EmailIdentityPolicy;
    /**
     * The email identity.
     */
    readonly emailIdentity: pulumi.Output<string>;
    /**
     * The text of the policy in JSON format.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * The name of the policy.
     */
    readonly policyName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a EmailIdentityPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EmailIdentityPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EmailIdentityPolicy resources.
 */
export interface EmailIdentityPolicyState {
    /**
     * The email identity.
     */
    emailIdentity?: pulumi.Input<string>;
    /**
     * The text of the policy in JSON format.
     */
    policy?: pulumi.Input<string>;
    /**
     * The name of the policy.
     */
    policyName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EmailIdentityPolicy resource.
 */
export interface EmailIdentityPolicyArgs {
    /**
     * The email identity.
     */
    emailIdentity: pulumi.Input<string>;
    /**
     * The text of the policy in JSON format.
     */
    policy: pulumi.Input<string>;
    /**
     * The name of the policy.
     */
    policyName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
