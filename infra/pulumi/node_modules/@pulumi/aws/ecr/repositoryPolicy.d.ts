import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
/**
 * Provides an Elastic Container Registry Repository Policy.
 *
 * Note that currently only one policy may be applied to a repository.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleRepository = new aws.ecr.Repository("example", {name: "example-repo"});
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
 * const exampleRepositoryPolicy = new aws.ecr.RepositoryPolicy("example", {
 *     repository: exampleRepository.name,
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
 * * `repository` - (String) Name of the ECR repository.
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * * `region` (String) Region where this resource is managed.
 *
 * Using `pulumi import`, import ECR Repository Policy using the repository name. For example:
 *
 * console
 *
 * % pulumi import aws_ecr_repository_policy.example example
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
    readonly repository: pulumi.Output<string>;
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
    policy?: pulumi.Input<string | inputs.ecr.PolicyDocument>;
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
    repository?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RepositoryPolicy resource.
 */
export interface RepositoryPolicyArgs {
    /**
     * The policy document. This is a JSON formatted string.
     */
    policy: pulumi.Input<string | inputs.ecr.PolicyDocument>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the repository to apply the policy.
     */
    repository: pulumi.Input<string>;
}
