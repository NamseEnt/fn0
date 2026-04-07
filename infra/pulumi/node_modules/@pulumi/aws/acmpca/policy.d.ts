import * as pulumi from "@pulumi/pulumi";
/**
 * Attaches a resource based policy to a private CA.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.iam.getPolicyDocument({
 *     statements: [
 *         {
 *             sid: "1",
 *             effect: "Allow",
 *             principals: [{
 *                 type: "AWS",
 *                 identifiers: [current.accountId],
 *             }],
 *             actions: [
 *                 "acm-pca:DescribeCertificateAuthority",
 *                 "acm-pca:GetCertificate",
 *                 "acm-pca:GetCertificateAuthorityCertificate",
 *                 "acm-pca:ListPermissions",
 *                 "acm-pca:ListTags",
 *             ],
 *             resources: [exampleAwsAcmpcaCertificateAuthority.arn],
 *         },
 *         {
 *             sid: "2",
 *             effect: allow,
 *             principals: [{
 *                 type: "AWS",
 *                 identifiers: [current.accountId],
 *             }],
 *             actions: ["acm-pca:IssueCertificate"],
 *             resources: [exampleAwsAcmpcaCertificateAuthority.arn],
 *             conditions: [{
 *                 test: "StringEquals",
 *                 variable: "acm-pca:TemplateArn",
 *                 values: ["arn:aws:acm-pca:::template/EndEntityCertificate/V1"],
 *             }],
 *         },
 *     ],
 * });
 * const examplePolicy = new aws.acmpca.Policy("example", {
 *     resourceArn: exampleAwsAcmpcaCertificateAuthority.arn,
 *     policy: example.then(example => example.json),
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the ACM PCA certificate authority.
 *
 * Using `pulumi import`, import `aws_acmpca_policy` using the `resource_arn` value. For example:
 *
 * console
 *
 * % pulumi import aws_acmpca_policy.example arn:aws:acm-pca:us-east-1:123456789012:certificate-authority/12345678-1234-1234-1234-123456789012
 */
export declare class Policy extends pulumi.CustomResource {
    /**
     * Get an existing Policy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PolicyState, opts?: pulumi.CustomResourceOptions): Policy;
    /**
     * Returns true if the given object is an instance of Policy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Policy;
    /**
     * JSON-formatted IAM policy to attach to the specified private CA resource.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the private CA to associate with the policy.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * Create a Policy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Policy resources.
 */
export interface PolicyState {
    /**
     * JSON-formatted IAM policy to attach to the specified private CA resource.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the private CA to associate with the policy.
     */
    resourceArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Policy resource.
 */
export interface PolicyArgs {
    /**
     * JSON-formatted IAM policy to attach to the specified private CA resource.
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the private CA to associate with the policy.
     */
    resourceArn: pulumi.Input<string>;
}
