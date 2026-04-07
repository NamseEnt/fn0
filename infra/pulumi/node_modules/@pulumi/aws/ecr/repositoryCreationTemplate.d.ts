import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Elastic Container Registry Repository Creation Template.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
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
 * const exampleRepositoryCreationTemplate = new aws.ecr.RepositoryCreationTemplate("example", {
 *     prefix: "example",
 *     description: "An example template",
 *     imageTagMutability: "IMMUTABLE",
 *     customRoleArn: "arn:aws:iam::123456789012:role/example",
 *     appliedFors: ["PULL_THROUGH_CACHE"],
 *     encryptionConfigurations: [{
 *         encryptionType: "AES256",
 *     }],
 *     repositoryPolicy: example.then(example => example.json),
 *     lifecyclePolicy: `{
 *   \\"rules\\": [
 *     {
 *       \\"rulePriority\\": 1,
 *       \\"description\\": \\"Expire images older than 14 days\\",
 *       \\"selection\\": {
 *         \\"tagStatus\\": \\"untagged\\",
 *         \\"countType\\": \\"sinceImagePushed\\",
 *         \\"countUnit\\": \\"days\\",
 *         \\"countNumber\\": 14
 *       },
 *       \\"action\\": {
 *         \\"type\\": \\"expire\\"
 *       }
 *     }
 *   ]
 * }
 * `,
 *     resourceTags: {
 *         Foo: "Bar",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import the ECR Repository Creating Templates using the `prefix`. For example:
 *
 * ```sh
 * $ pulumi import aws:ecr/repositoryCreationTemplate:RepositoryCreationTemplate example example
 * ```
 */
export declare class RepositoryCreationTemplate extends pulumi.CustomResource {
    /**
     * Get an existing RepositoryCreationTemplate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RepositoryCreationTemplateState, opts?: pulumi.CustomResourceOptions): RepositoryCreationTemplate;
    /**
     * Returns true if the given object is an instance of RepositoryCreationTemplate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RepositoryCreationTemplate;
    /**
     * Which features this template applies to. Must contain one or more of `PULL_THROUGH_CACHE` or `REPLICATION`.
     */
    readonly appliedFors: pulumi.Output<string[]>;
    /**
     * A custom IAM role to use for repository creation. Required if using repository tags or KMS encryption.
     */
    readonly customRoleArn: pulumi.Output<string | undefined>;
    /**
     * The description for this template.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Encryption configuration for any created repositories. See below for schema.
     */
    readonly encryptionConfigurations: pulumi.Output<outputs.ecr.RepositoryCreationTemplateEncryptionConfiguration[] | undefined>;
    /**
     * The tag mutability setting for any created repositories. Must be one of: `MUTABLE` or `IMMUTABLE`. Defaults to `MUTABLE`.
     */
    readonly imageTagMutability: pulumi.Output<string | undefined>;
    /**
     * Configuration block that defines filters to specify which image tags can override the default tag mutability setting. Only applicable when `imageTagMutability` is set to `IMMUTABLE_WITH_EXCLUSION` or `MUTABLE_WITH_EXCLUSION`. See below for schema.
     */
    readonly imageTagMutabilityExclusionFilters: pulumi.Output<outputs.ecr.RepositoryCreationTemplateImageTagMutabilityExclusionFilter[] | undefined>;
    /**
     * The lifecycle policy document to apply to any created repositories. See more details about [Policy Parameters](http://docs.aws.amazon.com/AmazonECR/latest/userguide/LifecyclePolicies.html#lifecycle_policy_parameters) in the official AWS docs. Consider using the `aws.ecr.getLifecyclePolicyDocument` dataSource to generate/manage the JSON document used for the `lifecyclePolicy` argument.
     */
    readonly lifecyclePolicy: pulumi.Output<string | undefined>;
    /**
     * The repository name prefix to match against. Use `ROOT` to match any prefix that doesn't explicitly match another template.
     */
    readonly prefix: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The registry ID the repository creation template applies to.
     */
    readonly registryId: pulumi.Output<string>;
    readonly repositoryPolicy: pulumi.Output<string | undefined>;
    /**
     * A map of tags to assign to any created repositories.
     */
    readonly resourceTags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Create a RepositoryCreationTemplate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RepositoryCreationTemplateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RepositoryCreationTemplate resources.
 */
export interface RepositoryCreationTemplateState {
    /**
     * Which features this template applies to. Must contain one or more of `PULL_THROUGH_CACHE` or `REPLICATION`.
     */
    appliedFors?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A custom IAM role to use for repository creation. Required if using repository tags or KMS encryption.
     */
    customRoleArn?: pulumi.Input<string>;
    /**
     * The description for this template.
     */
    description?: pulumi.Input<string>;
    /**
     * Encryption configuration for any created repositories. See below for schema.
     */
    encryptionConfigurations?: pulumi.Input<pulumi.Input<inputs.ecr.RepositoryCreationTemplateEncryptionConfiguration>[]>;
    /**
     * The tag mutability setting for any created repositories. Must be one of: `MUTABLE` or `IMMUTABLE`. Defaults to `MUTABLE`.
     */
    imageTagMutability?: pulumi.Input<string>;
    /**
     * Configuration block that defines filters to specify which image tags can override the default tag mutability setting. Only applicable when `imageTagMutability` is set to `IMMUTABLE_WITH_EXCLUSION` or `MUTABLE_WITH_EXCLUSION`. See below for schema.
     */
    imageTagMutabilityExclusionFilters?: pulumi.Input<pulumi.Input<inputs.ecr.RepositoryCreationTemplateImageTagMutabilityExclusionFilter>[]>;
    /**
     * The lifecycle policy document to apply to any created repositories. See more details about [Policy Parameters](http://docs.aws.amazon.com/AmazonECR/latest/userguide/LifecyclePolicies.html#lifecycle_policy_parameters) in the official AWS docs. Consider using the `aws.ecr.getLifecyclePolicyDocument` dataSource to generate/manage the JSON document used for the `lifecyclePolicy` argument.
     */
    lifecyclePolicy?: pulumi.Input<string>;
    /**
     * The repository name prefix to match against. Use `ROOT` to match any prefix that doesn't explicitly match another template.
     */
    prefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The registry ID the repository creation template applies to.
     */
    registryId?: pulumi.Input<string>;
    repositoryPolicy?: pulumi.Input<string>;
    /**
     * A map of tags to assign to any created repositories.
     */
    resourceTags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a RepositoryCreationTemplate resource.
 */
export interface RepositoryCreationTemplateArgs {
    /**
     * Which features this template applies to. Must contain one or more of `PULL_THROUGH_CACHE` or `REPLICATION`.
     */
    appliedFors: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A custom IAM role to use for repository creation. Required if using repository tags or KMS encryption.
     */
    customRoleArn?: pulumi.Input<string>;
    /**
     * The description for this template.
     */
    description?: pulumi.Input<string>;
    /**
     * Encryption configuration for any created repositories. See below for schema.
     */
    encryptionConfigurations?: pulumi.Input<pulumi.Input<inputs.ecr.RepositoryCreationTemplateEncryptionConfiguration>[]>;
    /**
     * The tag mutability setting for any created repositories. Must be one of: `MUTABLE` or `IMMUTABLE`. Defaults to `MUTABLE`.
     */
    imageTagMutability?: pulumi.Input<string>;
    /**
     * Configuration block that defines filters to specify which image tags can override the default tag mutability setting. Only applicable when `imageTagMutability` is set to `IMMUTABLE_WITH_EXCLUSION` or `MUTABLE_WITH_EXCLUSION`. See below for schema.
     */
    imageTagMutabilityExclusionFilters?: pulumi.Input<pulumi.Input<inputs.ecr.RepositoryCreationTemplateImageTagMutabilityExclusionFilter>[]>;
    /**
     * The lifecycle policy document to apply to any created repositories. See more details about [Policy Parameters](http://docs.aws.amazon.com/AmazonECR/latest/userguide/LifecyclePolicies.html#lifecycle_policy_parameters) in the official AWS docs. Consider using the `aws.ecr.getLifecyclePolicyDocument` dataSource to generate/manage the JSON document used for the `lifecyclePolicy` argument.
     */
    lifecyclePolicy?: pulumi.Input<string>;
    /**
     * The repository name prefix to match against. Use `ROOT` to match any prefix that doesn't explicitly match another template.
     */
    prefix: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    repositoryPolicy?: pulumi.Input<string>;
    /**
     * A map of tags to assign to any created repositories.
     */
    resourceTags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
