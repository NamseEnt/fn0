import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS X-Ray Resource Policy.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.xray.ResourcePolicy("test", {
 *     policyName: "test",
 *     policyDocument: "{\"Version\":\"2012-10-17\",\"Statement\":[{\"Sid\":\"AllowXRayAccess\",\"Effect\":\"Allow\",\"Principal\":{\"AWS\":\"*\"},\"Action\":[\"xray:*\",\"xray:PutResourcePolicy\"],\"Resource\":\"*\"}]}",
 *     bypassPolicyLockoutCheck: true,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import X-Ray Resource Policy using the `policy_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:xray/resourcePolicy:ResourcePolicy example resource_policy-name
 * ```
 */
export declare class ResourcePolicy extends pulumi.CustomResource {
    /**
     * Get an existing ResourcePolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResourcePolicyState, opts?: pulumi.CustomResourceOptions): ResourcePolicy;
    /**
     * Returns true if the given object is an instance of ResourcePolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResourcePolicy;
    /**
     * Flag to indicate whether to bypass the resource policy lockout safety check. Setting this value to true increases the risk that the policy becomes unmanageable. Do not set this value to true indiscriminately. Use this parameter only when you include a policy in the request and you intend to prevent the principal that is making the request from making a subsequent PutResourcePolicy request. The default value is `false`.
     */
    readonly bypassPolicyLockoutCheck: pulumi.Output<boolean | undefined>;
    /**
     * When the policy was last updated, in Unix time seconds.
     */
    readonly lastUpdatedTime: pulumi.Output<string>;
    /**
     * JSON string of the resource policy or resource policy document, which can be up to 5kb in size.
     *
     * The following arguments are optional:
     */
    readonly policyDocument: pulumi.Output<string>;
    /**
     * name of the resource policy. Must be unique within a specific Amazon Web Services account.
     */
    readonly policyName: pulumi.Output<string>;
    /**
     * Specifies a specific policy revision, to ensure an atomic create operation. By default the resource policy is created if it does not exist, or updated with an incremented revision id. The revision id is unique to each policy in the account. If the policy revision id does not match the latest revision id, the operation will fail with an InvalidPolicyRevisionIdException exception. You can also provide a PolicyRevisionId of 0. In this case, the operation will fail with an InvalidPolicyRevisionIdException exception if a resource policy with the same name already exists.
     */
    readonly policyRevisionId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a ResourcePolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ResourcePolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResourcePolicy resources.
 */
export interface ResourcePolicyState {
    /**
     * Flag to indicate whether to bypass the resource policy lockout safety check. Setting this value to true increases the risk that the policy becomes unmanageable. Do not set this value to true indiscriminately. Use this parameter only when you include a policy in the request and you intend to prevent the principal that is making the request from making a subsequent PutResourcePolicy request. The default value is `false`.
     */
    bypassPolicyLockoutCheck?: pulumi.Input<boolean>;
    /**
     * When the policy was last updated, in Unix time seconds.
     */
    lastUpdatedTime?: pulumi.Input<string>;
    /**
     * JSON string of the resource policy or resource policy document, which can be up to 5kb in size.
     *
     * The following arguments are optional:
     */
    policyDocument?: pulumi.Input<string>;
    /**
     * name of the resource policy. Must be unique within a specific Amazon Web Services account.
     */
    policyName?: pulumi.Input<string>;
    /**
     * Specifies a specific policy revision, to ensure an atomic create operation. By default the resource policy is created if it does not exist, or updated with an incremented revision id. The revision id is unique to each policy in the account. If the policy revision id does not match the latest revision id, the operation will fail with an InvalidPolicyRevisionIdException exception. You can also provide a PolicyRevisionId of 0. In this case, the operation will fail with an InvalidPolicyRevisionIdException exception if a resource policy with the same name already exists.
     */
    policyRevisionId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ResourcePolicy resource.
 */
export interface ResourcePolicyArgs {
    /**
     * Flag to indicate whether to bypass the resource policy lockout safety check. Setting this value to true increases the risk that the policy becomes unmanageable. Do not set this value to true indiscriminately. Use this parameter only when you include a policy in the request and you intend to prevent the principal that is making the request from making a subsequent PutResourcePolicy request. The default value is `false`.
     */
    bypassPolicyLockoutCheck?: pulumi.Input<boolean>;
    /**
     * JSON string of the resource policy or resource policy document, which can be up to 5kb in size.
     *
     * The following arguments are optional:
     */
    policyDocument: pulumi.Input<string>;
    /**
     * name of the resource policy. Must be unique within a specific Amazon Web Services account.
     */
    policyName: pulumi.Input<string>;
    /**
     * Specifies a specific policy revision, to ensure an atomic create operation. By default the resource policy is created if it does not exist, or updated with an incremented revision id. The revision id is unique to each policy in the account. If the policy revision id does not match the latest revision id, the operation will fail with an InvalidPolicyRevisionIdException exception. You can also provide a PolicyRevisionId of 0. In this case, the operation will fail with an InvalidPolicyRevisionIdException exception if a resource policy with the same name already exists.
     */
    policyRevisionId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
