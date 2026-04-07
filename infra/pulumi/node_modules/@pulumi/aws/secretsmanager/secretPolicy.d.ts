import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage AWS Secrets Manager secret policy.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleSecret = new aws.secretsmanager.Secret("example", {name: "example"});
 * const example = aws.iam.getPolicyDocument({
 *     statements: [{
 *         sid: "EnableAnotherAWSAccountToReadTheSecret",
 *         effect: "Allow",
 *         principals: [{
 *             type: "AWS",
 *             identifiers: ["arn:aws:iam::123456789012:root"],
 *         }],
 *         actions: ["secretsmanager:GetSecretValue"],
 *         resources: ["*"],
 *     }],
 * });
 * const exampleSecretPolicy = new aws.secretsmanager.SecretPolicy("example", {
 *     secretArn: exampleSecret.arn,
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
 * - `arn` (String) Amazon Resource Name (ARN) of the Secrets Manager secret.
 *
 * Using `pulumi import`, import `aws_secretsmanager_secret_policy` using the secret Amazon Resource Name (ARN). For example:
 *
 * console
 *
 * % pulumi import aws_secretsmanager_secret_policy.example arn:aws:secretsmanager:us-east-1:123456789012:secret:example-123456
 */
export declare class SecretPolicy extends pulumi.CustomResource {
    /**
     * Get an existing SecretPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SecretPolicyState, opts?: pulumi.CustomResourceOptions): SecretPolicy;
    /**
     * Returns true if the given object is an instance of SecretPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SecretPolicy;
    /**
     * Makes an optional API call to Zelkova to validate the Resource Policy to prevent broad access to your secret.
     */
    readonly blockPublicPolicy: pulumi.Output<boolean | undefined>;
    /**
     * Valid JSON document representing a [resource policy](https://docs.aws.amazon.com/secretsmanager/latest/userguide/auth-and-access_resource-based-policies.html). Unlike `aws.secretsmanager.Secret`, where `policy` can be set to `"{}"` to delete the policy, `"{}"` is not a valid policy since `policy` is required.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Secret ARN.
     *
     * The following arguments are optional:
     */
    readonly secretArn: pulumi.Output<string>;
    /**
     * Create a SecretPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: SecretPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SecretPolicy resources.
 */
export interface SecretPolicyState {
    /**
     * Makes an optional API call to Zelkova to validate the Resource Policy to prevent broad access to your secret.
     */
    blockPublicPolicy?: pulumi.Input<boolean>;
    /**
     * Valid JSON document representing a [resource policy](https://docs.aws.amazon.com/secretsmanager/latest/userguide/auth-and-access_resource-based-policies.html). Unlike `aws.secretsmanager.Secret`, where `policy` can be set to `"{}"` to delete the policy, `"{}"` is not a valid policy since `policy` is required.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Secret ARN.
     *
     * The following arguments are optional:
     */
    secretArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SecretPolicy resource.
 */
export interface SecretPolicyArgs {
    /**
     * Makes an optional API call to Zelkova to validate the Resource Policy to prevent broad access to your secret.
     */
    blockPublicPolicy?: pulumi.Input<boolean>;
    /**
     * Valid JSON document representing a [resource policy](https://docs.aws.amazon.com/secretsmanager/latest/userguide/auth-and-access_resource-based-policies.html). Unlike `aws.secretsmanager.Secret`, where `policy` can be set to `"{}"` to delete the policy, `"{}"` is not a valid policy since `policy` is required.
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Secret ARN.
     *
     * The following arguments are optional:
     */
    secretArn: pulumi.Input<string>;
}
