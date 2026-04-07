import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Elastic Container Registry Public Repository Policy.
 *
 * Note that currently only one policy may be applied to a repository.
 *
 * > **NOTE:** This resource can only be used in the `us-east-1` region.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleRepository = new aws.ecrpublic.Repository("example", {repositoryName: "example"});
 * const example = aws.iam.getPolicyDocument({
 *     statements: [{
 *         sid: "new policy",
 *         effect: "Allow",
 *         principals: [{
 *             type: "AWS",
 *             identifiers: ["123456789012"],
 *         }],
 *         actions: [
 *             "ecr:GetDownloadUrlForLayer",
 *             "ecr:BatchGetImage",
 *             "ecr:BatchCheckLayerAvailability",
 *             "ecr:PutImage",
 *             "ecr:InitiateLayerUpload",
 *             "ecr:UploadLayerPart",
 *             "ecr:CompleteLayerUpload",
 *             "ecr:DescribeRepositories",
 *             "ecr:GetRepositoryPolicy",
 *             "ecr:ListImages",
 *             "ecr:DeleteRepository",
 *             "ecr:BatchDeleteImage",
 *             "ecr:SetRepositoryPolicy",
 *             "ecr:DeleteRepositoryPolicy",
 *         ],
 *     }],
 * });
 * const exampleRepositoryPolicy = new aws.ecrpublic.RepositoryPolicy("example", {
 *     repositoryName: exampleRepository.repositoryName,
 *     policy: example.then(example => example.json),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import ECR Public Repository Policy using the repository name. For example:
 *
 * ```sh
 * $ pulumi import aws:ecrpublic/repositoryPolicy:RepositoryPolicy example example
 * ```
 */
export declare class RepositoryPolicy extends pulumi.CustomResource {
    /**
     * Get an existing RepositoryPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RepositoryPolicyState, opts?: pulumi.CustomResourceOptions): RepositoryPolicy;
    /**
     * Returns true if the given object is an instance of RepositoryPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RepositoryPolicy;
    /**
     * The policy document. This is a JSON formatted string.
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The registry ID where the repository was created.
     */
    readonly registryId: pulumi.Output<string>;
    /**
     * Name of the repository to apply the policy.
     */
    readonly repositoryName: pulumi.Output<string>;
    /**
     * Create a RepositoryPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RepositoryPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RepositoryPolicy resources.
 */
export interface RepositoryPolicyState {
    /**
     * The policy document. This is a JSON formatted string.
     */
    policy?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The registry ID where the repository was created.
     */
    registryId?: pulumi.Input<string>;
    /**
     * Name of the repository to apply the policy.
     */
    repositoryName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RepositoryPolicy resource.
 */
export interface RepositoryPolicyArgs {
    /**
     * The policy document. This is a JSON formatted string.
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the repository to apply the policy.
     */
    repositoryName: pulumi.Input<string>;
}
