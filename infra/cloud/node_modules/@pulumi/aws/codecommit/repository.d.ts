import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a CodeCommit Repository Resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.codecommit.Repository("test", {
 *     repositoryName: "MyTestRepository",
 *     description: "This is the Sample App Repository",
 * });
 * ```
 *
 * ### AWS KMS Customer Managed Keys (CMK)
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testKey = new aws.kms.Key("test", {
 *     description: "test",
 *     deletionWindowInDays: 7,
 * });
 * const test = new aws.codecommit.Repository("test", {
 *     repositoryName: "MyTestRepository",
 *     description: "This is the Sample App Repository",
 *     kmsKeyId: testKey.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CodeCommit repository using repository name. For example:
 *
 * ```sh
 * $ pulumi import aws:codecommit/repository:Repository imported ExistingRepo
 * ```
 */
export declare class Repository extends pulumi.CustomResource {
    /**
     * Get an existing Repository resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RepositoryState, opts?: pulumi.CustomResourceOptions): Repository;
    /**
     * Returns true if the given object is an instance of Repository.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Repository;
    /**
     * The ARN of the repository
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The URL to use for cloning the repository over HTTPS.
     */
    readonly cloneUrlHttp: pulumi.Output<string>;
    /**
     * The URL to use for cloning the repository over SSH.
     */
    readonly cloneUrlSsh: pulumi.Output<string>;
    /**
     * The default branch of the repository. The branch specified here needs to exist.
     */
    readonly defaultBranch: pulumi.Output<string | undefined>;
    /**
     * The description of the repository. This needs to be less than 1000 characters
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The ARN of the encryption key. If no key is specified, the default `aws/codecommit` Amazon Web Services managed key is used.
     */
    readonly kmsKeyId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of the repository
     */
    readonly repositoryId: pulumi.Output<string>;
    /**
     * The name for the repository. This needs to be less than 100 characters.
     */
    readonly repositoryName: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Repository resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RepositoryArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Repository resources.
 */
export interface RepositoryState {
    /**
     * The ARN of the repository
     */
    arn?: pulumi.Input<string>;
    /**
     * The URL to use for cloning the repository over HTTPS.
     */
    cloneUrlHttp?: pulumi.Input<string>;
    /**
     * The URL to use for cloning the repository over SSH.
     */
    cloneUrlSsh?: pulumi.Input<string>;
    /**
     * The default branch of the repository. The branch specified here needs to exist.
     */
    defaultBranch?: pulumi.Input<string>;
    /**
     * The description of the repository. This needs to be less than 1000 characters
     */
    description?: pulumi.Input<string>;
    /**
     * The ARN of the encryption key. If no key is specified, the default `aws/codecommit` Amazon Web Services managed key is used.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the repository
     */
    repositoryId?: pulumi.Input<string>;
    /**
     * The name for the repository. This needs to be less than 100 characters.
     */
    repositoryName?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Repository resource.
 */
export interface RepositoryArgs {
    /**
     * The default branch of the repository. The branch specified here needs to exist.
     */
    defaultBranch?: pulumi.Input<string>;
    /**
     * The description of the repository. This needs to be less than 1000 characters
     */
    description?: pulumi.Input<string>;
    /**
     * The ARN of the encryption key. If no key is specified, the default `aws/codecommit` Amazon Web Services managed key is used.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name for the repository. This needs to be less than 100 characters.
     */
    repositoryName: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
