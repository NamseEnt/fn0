import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an SNS data protection topic policy resource
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sns.Topic("example", {name: "example"});
 * const exampleDataProtectionPolicy = new aws.sns.DataProtectionPolicy("example", {
 *     arn: example.arn,
 *     policy: JSON.stringify({
 *         Description: "Example data protection policy",
 *         Name: "__example_data_protection_policy",
 *         Statement: [{
 *             DataDirection: "Inbound",
 *             DataIdentifier: ["arn:aws:dataprotection::aws:data-identifier/EmailAddress"],
 *             Operation: {
 *                 Deny: {},
 *             },
 *             Principal: ["*"],
 *             Sid: "__deny_statement_11ba9d96",
 *         }],
 *         Version: "2021-06-01",
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the SNS topic.
 *
 * Using `pulumi import`, import SNS Data Protection Topic Policy using the topic ARN. For example:
 *
 * console
 *
 * % pulumi import aws_sns_topic_data_protection_policy.example arn:aws:sns:us-west-2:123456789012:example
 */
export declare class DataProtectionPolicy extends pulumi.CustomResource {
    /**
     * Get an existing DataProtectionPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DataProtectionPolicyState, opts?: pulumi.CustomResourceOptions): DataProtectionPolicy;
    /**
     * Returns true if the given object is an instance of DataProtectionPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DataProtectionPolicy;
    /**
     * The ARN of the SNS topic
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The fully-formed AWS policy as JSON. For more information about building AWS IAM policy documents with this provider, see the AWS IAM Policy Document Guide.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a DataProtectionPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DataProtectionPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DataProtectionPolicy resources.
 */
export interface DataProtectionPolicyState {
    /**
     * The ARN of the SNS topic
     */
    arn?: pulumi.Input<string>;
    /**
     * The fully-formed AWS policy as JSON. For more information about building AWS IAM policy documents with this provider, see the AWS IAM Policy Document Guide.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DataProtectionPolicy resource.
 */
export interface DataProtectionPolicyArgs {
    /**
     * The ARN of the SNS topic
     */
    arn: pulumi.Input<string>;
    /**
     * The fully-formed AWS policy as JSON. For more information about building AWS IAM policy documents with this provider, see the AWS IAM Policy Document Guide.
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
