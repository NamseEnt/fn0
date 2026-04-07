import * as pulumi from "@pulumi/pulumi";
/**
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getInstances({});
 * const examplePermissionSet = new aws.ssoadmin.PermissionSet("example", {
 *     name: "Example",
 *     instanceArn: example.then(example => example.arns?.[0]),
 * });
 * const exampleGetPolicyDocument = aws.iam.getPolicyDocument({
 *     statements: [{
 *         sid: "1",
 *         actions: [
 *             "s3:ListAllMyBuckets",
 *             "s3:GetBucketLocation",
 *         ],
 *         resources: ["arn:aws:s3:::*"],
 *     }],
 * });
 * const examplePermissionSetInlinePolicy = new aws.ssoadmin.PermissionSetInlinePolicy("example", {
 *     inlinePolicy: exampleGetPolicyDocument.then(exampleGetPolicyDocument => exampleGetPolicyDocument.json),
 *     instanceArn: example.then(example => example.arns?.[0]),
 *     permissionSetArn: examplePermissionSet.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SSO Permission Set Inline Policies using the `permission_set_arn` and `instance_arn` separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:ssoadmin/permissionSetInlinePolicy:PermissionSetInlinePolicy example arn:aws:sso:::permissionSet/ssoins-2938j0x8920sbj72/ps-80383020jr9302rk,arn:aws:sso:::instance/ssoins-2938j0x8920sbj72
 * ```
 */
export declare class PermissionSetInlinePolicy extends pulumi.CustomResource {
    /**
     * Get an existing PermissionSetInlinePolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PermissionSetInlinePolicyState, opts?: pulumi.CustomResourceOptions): PermissionSetInlinePolicy;
    /**
     * Returns true if the given object is an instance of PermissionSetInlinePolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PermissionSetInlinePolicy;
    /**
     * The IAM inline policy to attach to a Permission Set.
     */
    readonly inlinePolicy: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the SSO Instance under which the operation will be executed.
     */
    readonly instanceArn: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the Permission Set.
     */
    readonly permissionSetArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a PermissionSetInlinePolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PermissionSetInlinePolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PermissionSetInlinePolicy resources.
 */
export interface PermissionSetInlinePolicyState {
    /**
     * The IAM inline policy to attach to a Permission Set.
     */
    inlinePolicy?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the SSO Instance under which the operation will be executed.
     */
    instanceArn?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the Permission Set.
     */
    permissionSetArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PermissionSetInlinePolicy resource.
 */
export interface PermissionSetInlinePolicyArgs {
    /**
     * The IAM inline policy to attach to a Permission Set.
     */
    inlinePolicy: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the SSO Instance under which the operation will be executed.
     */
    instanceArn: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the Permission Set.
     */
    permissionSetArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
