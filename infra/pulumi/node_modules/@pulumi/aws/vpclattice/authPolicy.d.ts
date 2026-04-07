import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS VPC Lattice Auth Policy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpclattice.Service("example", {
 *     name: "example-vpclattice-service",
 *     authType: "AWS_IAM",
 *     customDomainName: "example.com",
 * });
 * const exampleAuthPolicy = new aws.vpclattice.AuthPolicy("example", {
 *     resourceIdentifier: example.arn,
 *     policy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Action: "*",
 *             Effect: "Allow",
 *             Principal: "*",
 *             Resource: "*",
 *             Condition: {
 *                 StringNotEqualsIgnoreCase: {
 *                     "aws:PrincipalType": "anonymous",
 *                 },
 *             },
 *         }],
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC Lattice Auth Policy using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:vpclattice/authPolicy:AuthPolicy example abcd-12345678
 * ```
 */
export declare class AuthPolicy extends pulumi.CustomResource {
    /**
     * Get an existing AuthPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AuthPolicyState, opts?: pulumi.CustomResourceOptions): AuthPolicy;
    /**
     * Returns true if the given object is an instance of AuthPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AuthPolicy;
    /**
     * The auth policy. The policy string in JSON must not contain newlines or blank lines.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID or Amazon Resource Name (ARN) of the service network or service for which the policy is created.
     */
    readonly resourceIdentifier: pulumi.Output<string>;
    /**
     * The state of the auth policy. The auth policy is only active when the auth type is set to `AWS_IAM`. If you provide a policy, then authentication and authorization decisions are made based on this policy and the client's IAM policy. If the Auth type is `NONE`, then, any auth policy you provide will remain inactive.
     */
    readonly state: pulumi.Output<string | undefined>;
    /**
     * Create a AuthPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AuthPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AuthPolicy resources.
 */
export interface AuthPolicyState {
    /**
     * The auth policy. The policy string in JSON must not contain newlines or blank lines.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID or Amazon Resource Name (ARN) of the service network or service for which the policy is created.
     */
    resourceIdentifier?: pulumi.Input<string>;
    /**
     * The state of the auth policy. The auth policy is only active when the auth type is set to `AWS_IAM`. If you provide a policy, then authentication and authorization decisions are made based on this policy and the client's IAM policy. If the Auth type is `NONE`, then, any auth policy you provide will remain inactive.
     */
    state?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AuthPolicy resource.
 */
export interface AuthPolicyArgs {
    /**
     * The auth policy. The policy string in JSON must not contain newlines or blank lines.
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID or Amazon Resource Name (ARN) of the service network or service for which the policy is created.
     */
    resourceIdentifier: pulumi.Input<string>;
    /**
     * The state of the auth policy. The auth policy is only active when the auth type is set to `AWS_IAM`. If you provide a policy, then authentication and authorization decisions are made based on this policy and the client's IAM policy. If the Auth type is `NONE`, then, any auth policy you provide will remain inactive.
     */
    state?: pulumi.Input<string>;
}
