import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Elastic File System (EFS) File System Policy resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const fs = new aws.efs.FileSystem("fs", {creationToken: "my-product"});
 * const policy = aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         sid: "ExampleStatement01",
 *         effect: "Allow",
 *         principals: [{
 *             type: "AWS",
 *             identifiers: ["*"],
 *         }],
 *         actions: [
 *             "elasticfilesystem:ClientMount",
 *             "elasticfilesystem:ClientWrite",
 *         ],
 *         resources: [fs.arn],
 *         conditions: [{
 *             test: "Bool",
 *             variable: "aws:SecureTransport",
 *             values: ["true"],
 *         }],
 *     }],
 * });
 * const policyFileSystemPolicy = new aws.efs.FileSystemPolicy("policy", {
 *     fileSystemId: fs.id,
 *     policy: policy.apply(policy => policy.json),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import the EFS file system policies using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:efs/fileSystemPolicy:FileSystemPolicy foo fs-6fa144c6
 * ```
 */
export declare class FileSystemPolicy extends pulumi.CustomResource {
    /**
     * Get an existing FileSystemPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FileSystemPolicyState, opts?: pulumi.CustomResourceOptions): FileSystemPolicy;
    /**
     * Returns true if the given object is an instance of FileSystemPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is FileSystemPolicy;
    /**
     * A flag to indicate whether to bypass the `aws.efs.FileSystemPolicy` lockout safety check. The policy lockout safety check determines whether the policy in the request will prevent the principal making the request will be locked out from making future `PutFileSystemPolicy` requests on the file system. Set `bypassPolicyLockoutSafetyCheck` to `true` only when you intend to prevent the principal that is making the request from making a subsequent `PutFileSystemPolicy` request on the file system. The default value is `false`.
     */
    readonly bypassPolicyLockoutSafetyCheck: pulumi.Output<boolean | undefined>;
    /**
     * The ID of the EFS file system.
     */
    readonly fileSystemId: pulumi.Output<string>;
    /**
     * The JSON formatted file system policy for the EFS file system. see [Docs](https://docs.aws.amazon.com/efs/latest/ug/access-control-overview.html#access-control-manage-access-intro-resource-policies) for more info.
     *
     * The following arguments are optional:
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a FileSystemPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FileSystemPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering FileSystemPolicy resources.
 */
export interface FileSystemPolicyState {
    /**
     * A flag to indicate whether to bypass the `aws.efs.FileSystemPolicy` lockout safety check. The policy lockout safety check determines whether the policy in the request will prevent the principal making the request will be locked out from making future `PutFileSystemPolicy` requests on the file system. Set `bypassPolicyLockoutSafetyCheck` to `true` only when you intend to prevent the principal that is making the request from making a subsequent `PutFileSystemPolicy` request on the file system. The default value is `false`.
     */
    bypassPolicyLockoutSafetyCheck?: pulumi.Input<boolean>;
    /**
     * The ID of the EFS file system.
     */
    fileSystemId?: pulumi.Input<string>;
    /**
     * The JSON formatted file system policy for the EFS file system. see [Docs](https://docs.aws.amazon.com/efs/latest/ug/access-control-overview.html#access-control-manage-access-intro-resource-policies) for more info.
     *
     * The following arguments are optional:
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a FileSystemPolicy resource.
 */
export interface FileSystemPolicyArgs {
    /**
     * A flag to indicate whether to bypass the `aws.efs.FileSystemPolicy` lockout safety check. The policy lockout safety check determines whether the policy in the request will prevent the principal making the request will be locked out from making future `PutFileSystemPolicy` requests on the file system. Set `bypassPolicyLockoutSafetyCheck` to `true` only when you intend to prevent the principal that is making the request from making a subsequent `PutFileSystemPolicy` request on the file system. The default value is `false`.
     */
    bypassPolicyLockoutSafetyCheck?: pulumi.Input<boolean>;
    /**
     * The ID of the EFS file system.
     */
    fileSystemId: pulumi.Input<string>;
    /**
     * The JSON formatted file system policy for the EFS file system. see [Docs](https://docs.aws.amazon.com/efs/latest/ug/access-control-overview.html#access-control-manage-access-intro-resource-policies) for more info.
     *
     * The following arguments are optional:
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
