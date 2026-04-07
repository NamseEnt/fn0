import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a SES Identity Policy. More information about SES Sending Authorization Policies can be found in the [SES Developer Guide](https://docs.aws.amazon.com/ses/latest/DeveloperGuide/sending-authorization-policies.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleDomainIdentity = new aws.ses.DomainIdentity("example", {domain: "example.com"});
 * const example = aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         actions: [
 *             "SES:SendEmail",
 *             "SES:SendRawEmail",
 *         ],
 *         resources: [exampleDomainIdentity.arn],
 *         principals: [{
 *             identifiers: ["*"],
 *             type: "AWS",
 *         }],
 *     }],
 * });
 * const exampleIdentityPolicy = new aws.ses.IdentityPolicy("example", {
 *     identity: exampleDomainIdentity.arn,
 *     name: "example",
 *     policy: example.apply(example => example.json),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SES Identity Policies using the identity and policy name, separated by a pipe character (`|`). For example:
 *
 * ```sh
 * $ pulumi import aws:ses/identityPolicy:IdentityPolicy example 'example.com|example'
 * ```
 */
export declare class IdentityPolicy extends pulumi.CustomResource {
    /**
     * Get an existing IdentityPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: IdentityPolicyState, opts?: pulumi.CustomResourceOptions): IdentityPolicy;
    /**
     * Returns true if the given object is an instance of IdentityPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is IdentityPolicy;
    /**
     * Name or Amazon Resource Name (ARN) of the SES Identity.
     */
    readonly identity: pulumi.Output<string>;
    /**
     * Name of the policy.
     */
    readonly name: pulumi.Output<string>;
    /**
     * JSON string of the policy.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a IdentityPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: IdentityPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering IdentityPolicy resources.
 */
export interface IdentityPolicyState {
    /**
     * Name or Amazon Resource Name (ARN) of the SES Identity.
     */
    identity?: pulumi.Input<string>;
    /**
     * Name of the policy.
     */
    name?: pulumi.Input<string>;
    /**
     * JSON string of the policy.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a IdentityPolicy resource.
 */
export interface IdentityPolicyArgs {
    /**
     * Name or Amazon Resource Name (ARN) of the SES Identity.
     */
    identity: pulumi.Input<string>;
    /**
     * Name of the policy.
     */
    name?: pulumi.Input<string>;
    /**
     * JSON string of the policy.
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
