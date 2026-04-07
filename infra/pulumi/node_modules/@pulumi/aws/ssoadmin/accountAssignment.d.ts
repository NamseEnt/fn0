import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Single Sign-On (SSO) Account Assignment resource
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getInstances({});
 * const exampleGetPermissionSet = example.then(example => aws.ssoadmin.getPermissionSet({
 *     instanceArn: example.arns?.[0],
 *     name: "AWSReadOnlyAccess",
 * }));
 * const exampleGetGroup = example.then(example => aws.identitystore.getGroup({
 *     identityStoreId: example.identityStoreIds?.[0],
 *     alternateIdentifier: {
 *         uniqueAttribute: {
 *             attributePath: "DisplayName",
 *             attributeValue: "ExampleGroup",
 *         },
 *     },
 * }));
 * const exampleAccountAssignment = new aws.ssoadmin.AccountAssignment("example", {
 *     instanceArn: example.then(example => example.arns?.[0]),
 *     permissionSetArn: exampleGetPermissionSet.then(exampleGetPermissionSet => exampleGetPermissionSet.arn),
 *     principalId: exampleGetGroup.then(exampleGetGroup => exampleGetGroup.groupId),
 *     principalType: "GROUP",
 *     targetId: "123456789012",
 *     targetType: "AWS_ACCOUNT",
 * });
 * ```
 *
 * ### With Managed Policy Attachment
 *
 * > Because destruction of a managed policy attachment resource also re-provisions the associated permission set to all accounts, explicitly indicating the dependency with the account assignment resource via the `dependsOn` meta argument is necessary to ensure proper deletion order when these resources are used together.
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
 * const exampleGroup = new aws.identitystore.Group("example", {
 *     identityStoreId: example.then(example => example.identityStoreIds?.[0]),
 *     displayName: "Admin",
 *     description: "Admin Group",
 * });
 * const exampleAccountAssignment = new aws.ssoadmin.AccountAssignment("example", {
 *     instanceArn: example.then(example => example.arns?.[0]),
 *     permissionSetArn: examplePermissionSet.arn,
 *     principalId: exampleGroup.groupId,
 *     principalType: "GROUP",
 *     targetId: "123456789012",
 *     targetType: "AWS_ACCOUNT",
 * });
 * const exampleManagedPolicyAttachment = new aws.ssoadmin.ManagedPolicyAttachment("example", {
 *     instanceArn: example.then(example => example.arns?.[0]),
 *     managedPolicyArn: "arn:aws:iam::aws:policy/AlexaForBusinessDeviceSetup",
 *     permissionSetArn: examplePermissionSet.arn,
 * }, {
 *     dependsOn: [exampleAccountAssignment],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SSO Account Assignments using the `principal_id`, `principal_type`, `target_id`, `target_type`, `permission_set_arn`, `instance_arn` separated by commas (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:ssoadmin/accountAssignment:AccountAssignment example f81d4fae-7dec-11d0-a765-00a0c91e6bf6,GROUP,1234567890,AWS_ACCOUNT,arn:aws:sso:::permissionSet/ssoins-0123456789abcdef/ps-0123456789abcdef,arn:aws:sso:::instance/ssoins-0123456789abcdef
 * ```
 */
export declare class AccountAssignment extends pulumi.CustomResource {
    /**
     * Get an existing AccountAssignment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AccountAssignmentState, opts?: pulumi.CustomResourceOptions): AccountAssignment;
    /**
     * Returns true if the given object is an instance of AccountAssignment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AccountAssignment;
    /**
     * The Amazon Resource Name (ARN) of the SSO Instance.
     */
    readonly instanceArn: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the Permission Set that the admin wants to grant the principal access to.
     */
    readonly permissionSetArn: pulumi.Output<string>;
    /**
     * An identifier for an object in SSO, such as a user or group. PrincipalIds are GUIDs (For example, `f81d4fae-7dec-11d0-a765-00a0c91e6bf6`).
     */
    readonly principalId: pulumi.Output<string>;
    /**
     * The entity type for which the assignment will be created. Valid values: `USER`, `GROUP`.
     */
    readonly principalType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * An AWS account identifier, typically a 10-12 digit string.
     */
    readonly targetId: pulumi.Output<string>;
    /**
     * The entity type for which the assignment will be created. Valid values: `AWS_ACCOUNT`.
     */
    readonly targetType: pulumi.Output<string | undefined>;
    /**
     * Create a AccountAssignment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AccountAssignmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AccountAssignment resources.
 */
export interface AccountAssignmentState {
    /**
     * The Amazon Resource Name (ARN) of the SSO Instance.
     */
    instanceArn?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the Permission Set that the admin wants to grant the principal access to.
     */
    permissionSetArn?: pulumi.Input<string>;
    /**
     * An identifier for an object in SSO, such as a user or group. PrincipalIds are GUIDs (For example, `f81d4fae-7dec-11d0-a765-00a0c91e6bf6`).
     */
    principalId?: pulumi.Input<string>;
    /**
     * The entity type for which the assignment will be created. Valid values: `USER`, `GROUP`.
     */
    principalType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * An AWS account identifier, typically a 10-12 digit string.
     */
    targetId?: pulumi.Input<string>;
    /**
     * The entity type for which the assignment will be created. Valid values: `AWS_ACCOUNT`.
     */
    targetType?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AccountAssignment resource.
 */
export interface AccountAssignmentArgs {
    /**
     * The Amazon Resource Name (ARN) of the SSO Instance.
     */
    instanceArn: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the Permission Set that the admin wants to grant the principal access to.
     */
    permissionSetArn: pulumi.Input<string>;
    /**
     * An identifier for an object in SSO, such as a user or group. PrincipalIds are GUIDs (For example, `f81d4fae-7dec-11d0-a765-00a0c91e6bf6`).
     */
    principalId: pulumi.Input<string>;
    /**
     * The entity type for which the assignment will be created. Valid values: `USER`, `GROUP`.
     */
    principalType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * An AWS account identifier, typically a 10-12 digit string.
     */
    targetId: pulumi.Input<string>;
    /**
     * The entity type for which the assignment will be created. Valid values: `AWS_ACCOUNT`.
     */
    targetType?: pulumi.Input<string>;
}
