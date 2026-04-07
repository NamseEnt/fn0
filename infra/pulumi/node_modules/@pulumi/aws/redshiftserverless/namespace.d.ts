import * as pulumi from "@pulumi/pulumi";
/**
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshiftserverless.Namespace("example", {namespaceName: "concurrency-scaling"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift Serverless Namespaces using the `namespace_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshiftserverless/namespace:Namespace example example
 * ```
 */
export declare class Namespace extends pulumi.CustomResource {
    /**
     * Get an existing Namespace resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: NamespaceState, opts?: pulumi.CustomResourceOptions): Namespace;
    /**
     * Returns true if the given object is an instance of Namespace.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Namespace;
    /**
     * Amazon Resource Name (ARN) of namespace's admin user credentials secret.
     */
    readonly adminPasswordSecretArn: pulumi.Output<string>;
    /**
     * ID of the KMS key used to encrypt the namespace's admin credentials secret.
     */
    readonly adminPasswordSecretKmsKeyId: pulumi.Output<string>;
    /**
     * The password of the administrator for the first database created in the namespace.
     * Conflicts with `manageAdminPassword` and `adminUserPasswordWo`.
     */
    readonly adminUserPassword: pulumi.Output<string | undefined>;
    /**
     * **NOTE:** This field is write-only and its value will not be updated in state as part of read operations.
     * The password of the administrator for the first database created in the namespace.
     * Conflicts with `manageAdminPassword` and `adminUserPassword`.
     */
    readonly adminUserPasswordWo: pulumi.Output<string | undefined>;
    /**
     * Used together with `adminUserPasswordWo` to trigger an update. Increment this value when an update to the `adminUserPasswordWo` is required
     */
    readonly adminUserPasswordWoVersion: pulumi.Output<number | undefined>;
    /**
     * The username of the administrator for the first database created in the namespace.
     */
    readonly adminUsername: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the Redshift Serverless Namespace.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The name of the first database created in the namespace.
     */
    readonly dbName: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the IAM role to set as a default in the namespace. When specifying `defaultIamRoleArn`, it also must be part of `iamRoles`.
     */
    readonly defaultIamRoleArn: pulumi.Output<string | undefined>;
    /**
     * A list of IAM roles to associate with the namespace.
     */
    readonly iamRoles: pulumi.Output<string[]>;
    /**
     * The ARN of the Amazon Web Services Key Management Service key used to encrypt your data.
     */
    readonly kmsKeyId: pulumi.Output<string>;
    /**
     * The types of logs the namespace can export. Available export types are `userlog`, `connectionlog`, and `useractivitylog`.
     */
    readonly logExports: pulumi.Output<string[] | undefined>;
    /**
     * Whether to use AWS SecretManager to manage namespace's admin credentials.
     * Conflicts with `adminUserPassword` and `adminUserPasswordWo`.
     */
    readonly manageAdminPassword: pulumi.Output<boolean | undefined>;
    /**
     * The Redshift Namespace ID.
     */
    readonly namespaceId: pulumi.Output<string>;
    /**
     * The name of the namespace.
     */
    readonly namespaceName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a Namespace resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: NamespaceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Namespace resources.
 */
export interface NamespaceState {
    /**
     * Amazon Resource Name (ARN) of namespace's admin user credentials secret.
     */
    adminPasswordSecretArn?: pulumi.Input<string>;
    /**
     * ID of the KMS key used to encrypt the namespace's admin credentials secret.
     */
    adminPasswordSecretKmsKeyId?: pulumi.Input<string>;
    /**
     * The password of the administrator for the first database created in the namespace.
     * Conflicts with `manageAdminPassword` and `adminUserPasswordWo`.
     */
    adminUserPassword?: pulumi.Input<string>;
    /**
     * **NOTE:** This field is write-only and its value will not be updated in state as part of read operations.
     * The password of the administrator for the first database created in the namespace.
     * Conflicts with `manageAdminPassword` and `adminUserPassword`.
     */
    adminUserPasswordWo?: pulumi.Input<string>;
    /**
     * Used together with `adminUserPasswordWo` to trigger an update. Increment this value when an update to the `adminUserPasswordWo` is required
     */
    adminUserPasswordWoVersion?: pulumi.Input<number>;
    /**
     * The username of the administrator for the first database created in the namespace.
     */
    adminUsername?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the Redshift Serverless Namespace.
     */
    arn?: pulumi.Input<string>;
    /**
     * The name of the first database created in the namespace.
     */
    dbName?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the IAM role to set as a default in the namespace. When specifying `defaultIamRoleArn`, it also must be part of `iamRoles`.
     */
    defaultIamRoleArn?: pulumi.Input<string>;
    /**
     * A list of IAM roles to associate with the namespace.
     */
    iamRoles?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ARN of the Amazon Web Services Key Management Service key used to encrypt your data.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The types of logs the namespace can export. Available export types are `userlog`, `connectionlog`, and `useractivitylog`.
     */
    logExports?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Whether to use AWS SecretManager to manage namespace's admin credentials.
     * Conflicts with `adminUserPassword` and `adminUserPasswordWo`.
     */
    manageAdminPassword?: pulumi.Input<boolean>;
    /**
     * The Redshift Namespace ID.
     */
    namespaceId?: pulumi.Input<string>;
    /**
     * The name of the namespace.
     */
    namespaceName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a Namespace resource.
 */
export interface NamespaceArgs {
    /**
     * ID of the KMS key used to encrypt the namespace's admin credentials secret.
     */
    adminPasswordSecretKmsKeyId?: pulumi.Input<string>;
    /**
     * The password of the administrator for the first database created in the namespace.
     * Conflicts with `manageAdminPassword` and `adminUserPasswordWo`.
     */
    adminUserPassword?: pulumi.Input<string>;
    /**
     * **NOTE:** This field is write-only and its value will not be updated in state as part of read operations.
     * The password of the administrator for the first database created in the namespace.
     * Conflicts with `manageAdminPassword` and `adminUserPassword`.
     */
    adminUserPasswordWo?: pulumi.Input<string>;
    /**
     * Used together with `adminUserPasswordWo` to trigger an update. Increment this value when an update to the `adminUserPasswordWo` is required
     */
    adminUserPasswordWoVersion?: pulumi.Input<number>;
    /**
     * The username of the administrator for the first database created in the namespace.
     */
    adminUsername?: pulumi.Input<string>;
    /**
     * The name of the first database created in the namespace.
     */
    dbName?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the IAM role to set as a default in the namespace. When specifying `defaultIamRoleArn`, it also must be part of `iamRoles`.
     */
    defaultIamRoleArn?: pulumi.Input<string>;
    /**
     * A list of IAM roles to associate with the namespace.
     */
    iamRoles?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The ARN of the Amazon Web Services Key Management Service key used to encrypt your data.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * The types of logs the namespace can export. Available export types are `userlog`, `connectionlog`, and `useractivitylog`.
     */
    logExports?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Whether to use AWS SecretManager to manage namespace's admin credentials.
     * Conflicts with `adminUserPassword` and `adminUserPasswordWo`.
     */
    manageAdminPassword?: pulumi.Input<boolean>;
    /**
     * The name of the namespace.
     */
    namespaceName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
