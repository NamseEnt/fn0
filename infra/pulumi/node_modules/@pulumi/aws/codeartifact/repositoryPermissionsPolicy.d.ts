import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a CodeArtifact Repostory Permissions Policy Resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleKey = new aws.kms.Key("example", {description: "domain key"});
 * const exampleDomain = new aws.codeartifact.Domain("example", {
 *     domain: "example",
 *     encryptionKey: exampleKey.arn,
 * });
 * const exampleRepository = new aws.codeartifact.Repository("example", {
 *     repository: "example",
 *     domain: exampleDomain.domain,
 * });
 * const example = aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "*",
 *             identifiers: ["*"],
 *         }],
 *         actions: ["codeartifact:ReadFromRepository"],
 *         resources: [exampleRepository.arn],
 *     }],
 * });
 * const exampleRepositoryPermissionsPolicy = new aws.codeartifact.RepositoryPermissionsPolicy("example", {
 *     repository: exampleRepository.repository,
 *     domain: exampleDomain.domain,
 *     policyDocument: example.apply(example => example.json),
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the CodeArtifact repository.
 *
 * Using `pulumi import`, import CodeArtifact Repository Permissions Policies using the CodeArtifact Repository ARN. For example:
 *
 * console
 *
 * % pulumi import aws_codeartifact_repository_permissions_policy.example arn:aws:codeartifact:us-west-2:012345678912:repository/tf-acc-test-6968272603913957763/tf-acc-test-6968272603913957763
 */
export declare class RepositoryPermissionsPolicy extends pulumi.CustomResource {
    /**
     * Get an existing RepositoryPermissionsPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RepositoryPermissionsPolicyState, opts?: pulumi.CustomResourceOptions): RepositoryPermissionsPolicy;
    /**
     * Returns true if the given object is an instance of RepositoryPermissionsPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RepositoryPermissionsPolicy;
    /**
     * The name of the domain on which to set the resource policy.
     */
    readonly domain: pulumi.Output<string>;
    /**
     * The account number of the AWS account that owns the domain.
     */
    readonly domainOwner: pulumi.Output<string>;
    /**
     * A JSON policy string to be set as the access control resource policy on the provided domain.
     */
    readonly policyDocument: pulumi.Output<string>;
    /**
     * The current revision of the resource policy to be set. This revision is used for optimistic locking, which prevents others from overwriting your changes to the domain's resource policy.
     */
    readonly policyRevision: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The name of the repository to set the resource policy on.
     */
    readonly repository: pulumi.Output<string>;
    /**
     * The ARN of the resource associated with the resource policy.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * Create a RepositoryPermissionsPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RepositoryPermissionsPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RepositoryPermissionsPolicy resources.
 */
export interface RepositoryPermissionsPolicyState {
    /**
     * The name of the domain on which to set the resource policy.
     */
    domain?: pulumi.Input<string>;
    /**
     * The account number of the AWS account that owns the domain.
     */
    domainOwner?: pulumi.Input<string>;
    /**
     * A JSON policy string to be set as the access control resource policy on the provided domain.
     */
    policyDocument?: pulumi.Input<string>;
    /**
     * The current revision of the resource policy to be set. This revision is used for optimistic locking, which prevents others from overwriting your changes to the domain's resource policy.
     */
    policyRevision?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the repository to set the resource policy on.
     */
    repository?: pulumi.Input<string>;
    /**
     * The ARN of the resource associated with the resource policy.
     */
    resourceArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RepositoryPermissionsPolicy resource.
 */
export interface RepositoryPermissionsPolicyArgs {
    /**
     * The name of the domain on which to set the resource policy.
     */
    domain: pulumi.Input<string>;
    /**
     * The account number of the AWS account that owns the domain.
     */
    domainOwner?: pulumi.Input<string>;
    /**
     * A JSON policy string to be set as the access control resource policy on the provided domain.
     */
    policyDocument: pulumi.Input<string>;
    /**
     * The current revision of the resource policy to be set. This revision is used for optimistic locking, which prevents others from overwriting your changes to the domain's resource policy.
     */
    policyRevision?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the repository to set the resource policy on.
     */
    repository: pulumi.Input<string>;
}
