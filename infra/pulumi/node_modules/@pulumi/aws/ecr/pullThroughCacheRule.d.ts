import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Elastic Container Registry Pull Through Cache Rule.
 *
 * More information about pull through cache rules, including the set of supported
 * upstream repositories, see [Using pull through cache rules](https://docs.aws.amazon.com/AmazonECR/latest/userguide/pull-through-cache.html).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ecr.PullThroughCacheRule("example", {
 *     ecrRepositoryPrefix: "ecr-public",
 *     upstreamRegistryUrl: "public.ecr.aws",
 *     credentialArn: "arn:aws:secretsmanager:us-east-1:123456789:secret:ecr-pullthroughcache/ecrpublic",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a pull-through cache rule using the `ecr_repository_prefix`. For example:
 *
 * ```sh
 * $ pulumi import aws:ecr/pullThroughCacheRule:PullThroughCacheRule example ecr-public
 * ```
 */
export declare class PullThroughCacheRule extends pulumi.CustomResource {
    /**
     * Get an existing PullThroughCacheRule resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PullThroughCacheRuleState, opts?: pulumi.CustomResourceOptions): PullThroughCacheRule;
    /**
     * Returns true if the given object is an instance of PullThroughCacheRule.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PullThroughCacheRule;
    /**
     * ARN of the Secret which will be used to authenticate against the registry.
     */
    readonly credentialArn: pulumi.Output<string | undefined>;
    /**
     * The ARN of the IAM role associated with the pull through cache rule. Must be specified if the upstream registry is a cross-account ECR private registry. See [AWS Document - Setting up permissions for cross-account ECR to ECR PTC](https://docs.aws.amazon.com/AmazonECR/latest/userguide/pull-through-cache-private.html).
     */
    readonly customRoleArn: pulumi.Output<string | undefined>;
    /**
     * The repository name prefix to use when caching images from the source registry. Use `ROOT` as the prefix to apply a template to all repositories in your registry that don't have an associated pull through cache rule.
     */
    readonly ecrRepositoryPrefix: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The registry ID where the repository was created.
     */
    readonly registryId: pulumi.Output<string>;
    /**
     * The registry URL of the upstream registry to use as the source.
     */
    readonly upstreamRegistryUrl: pulumi.Output<string>;
    /**
     * The upstream repository prefix associated with the pull through cache rule. Used if the upstream registry is an ECR private registry. If not specified, it's set to `ROOT`, which allows matching with any upstream repository. See [AWS Document - Customizing repository prefixes for ECR to ECR pull through cache](https://docs.aws.amazon.com/AmazonECR/latest/userguide/pull-through-cache-private-wildcards.html).
     */
    readonly upstreamRepositoryPrefix: pulumi.Output<string | undefined>;
    /**
     * Create a PullThroughCacheRule resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PullThroughCacheRuleArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PullThroughCacheRule resources.
 */
export interface PullThroughCacheRuleState {
    /**
     * ARN of the Secret which will be used to authenticate against the registry.
     */
    credentialArn?: pulumi.Input<string>;
    /**
     * The ARN of the IAM role associated with the pull through cache rule. Must be specified if the upstream registry is a cross-account ECR private registry. See [AWS Document - Setting up permissions for cross-account ECR to ECR PTC](https://docs.aws.amazon.com/AmazonECR/latest/userguide/pull-through-cache-private.html).
     */
    customRoleArn?: pulumi.Input<string>;
    /**
     * The repository name prefix to use when caching images from the source registry. Use `ROOT` as the prefix to apply a template to all repositories in your registry that don't have an associated pull through cache rule.
     */
    ecrRepositoryPrefix?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The registry ID where the repository was created.
     */
    registryId?: pulumi.Input<string>;
    /**
     * The registry URL of the upstream registry to use as the source.
     */
    upstreamRegistryUrl?: pulumi.Input<string>;
    /**
     * The upstream repository prefix associated with the pull through cache rule. Used if the upstream registry is an ECR private registry. If not specified, it's set to `ROOT`, which allows matching with any upstream repository. See [AWS Document - Customizing repository prefixes for ECR to ECR pull through cache](https://docs.aws.amazon.com/AmazonECR/latest/userguide/pull-through-cache-private-wildcards.html).
     */
    upstreamRepositoryPrefix?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PullThroughCacheRule resource.
 */
export interface PullThroughCacheRuleArgs {
    /**
     * ARN of the Secret which will be used to authenticate against the registry.
     */
    credentialArn?: pulumi.Input<string>;
    /**
     * The ARN of the IAM role associated with the pull through cache rule. Must be specified if the upstream registry is a cross-account ECR private registry. See [AWS Document - Setting up permissions for cross-account ECR to ECR PTC](https://docs.aws.amazon.com/AmazonECR/latest/userguide/pull-through-cache-private.html).
     */
    customRoleArn?: pulumi.Input<string>;
    /**
     * The repository name prefix to use when caching images from the source registry. Use `ROOT` as the prefix to apply a template to all repositories in your registry that don't have an associated pull through cache rule.
     */
    ecrRepositoryPrefix: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The registry URL of the upstream registry to use as the source.
     */
    upstreamRegistryUrl: pulumi.Input<string>;
    /**
     * The upstream repository prefix associated with the pull through cache rule. Used if the upstream registry is an ECR private registry. If not specified, it's set to `ROOT`, which allows matching with any upstream repository. See [AWS Document - Customizing repository prefixes for ECR to ECR pull through cache](https://docs.aws.amazon.com/AmazonECR/latest/userguide/pull-through-cache-private-wildcards.html).
     */
    upstreamRepositoryPrefix?: pulumi.Input<string>;
}
