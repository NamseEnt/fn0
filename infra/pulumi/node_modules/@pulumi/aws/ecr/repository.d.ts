import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Elastic Container Registry Repository.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const foo = new aws.ecr.Repository("foo", {
 *     name: "bar",
 *     imageTagMutability: "MUTABLE",
 *     imageScanningConfiguration: {
 *         scanOnPush: true,
 *     },
 * });
 * ```
 *
 * ### With Image Tag Mutability Exclusion
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ecr.Repository("example", {
 *     name: "example-repo",
 *     imageTagMutability: "IMMUTABLE_WITH_EXCLUSION",
 *     imageTagMutabilityExclusionFilters: [
 *         {
 *             filter: "latest*",
 *             filterType: "WILDCARD",
 *         },
 *         {
 *             filter: "dev-*",
 *             filterType: "WILDCARD",
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * * `name` - (String) Name of the ECR repository.
 *
 * #### Optional
 *
 * * `account_id` (String) AWS Account where this resource is managed.
 *
 * * `region` (String) Region where this resource is managed.
 *
 * Using `pulumi import`, import ECR Repositories using the `name`. For example:
 *
 * console
 *
 * % pulumi import aws_ecr_repository.service test-service
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
     * Full ARN of the repository.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Encryption configuration for the repository. See below for schema.
     */
    readonly encryptionConfigurations: pulumi.Output<outputs.ecr.RepositoryEncryptionConfiguration[] | undefined>;
    /**
     * If `true`, will delete the repository even if it contains images.
     * Defaults to `false`.
     */
    readonly forceDelete: pulumi.Output<boolean | undefined>;
    /**
     * Configuration block that defines image scanning configuration for the repository. By default, image scanning must be manually triggered. See the [ECR User Guide](https://docs.aws.amazon.com/AmazonECR/latest/userguide/image-scanning.html) for more information about image scanning.
     */
    readonly imageScanningConfiguration: pulumi.Output<outputs.ecr.RepositoryImageScanningConfiguration | undefined>;
    /**
     * The tag mutability setting for the repository. Must be one of: `MUTABLE`, `IMMUTABLE`, `IMMUTABLE_WITH_EXCLUSION`, or `MUTABLE_WITH_EXCLUSION`. Defaults to `MUTABLE`.
     */
    readonly imageTagMutability: pulumi.Output<string | undefined>;
    /**
     * Configuration block that defines filters to specify which image tags can override the default tag mutability setting. Only applicable when `imageTagMutability` is set to `IMMUTABLE_WITH_EXCLUSION` or `MUTABLE_WITH_EXCLUSION`. See below for schema.
     */
    readonly imageTagMutabilityExclusionFilters: pulumi.Output<outputs.ecr.RepositoryImageTagMutabilityExclusionFilter[] | undefined>;
    /**
     * Name of the repository.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The registry ID where the repository was created.
     */
    readonly registryId: pulumi.Output<string>;
    /**
     * The URL of the repository (in the form `aws_account_id.dkr.ecr.region.amazonaws.com/repositoryName`).
     */
    readonly repositoryUrl: pulumi.Output<string>;
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
     * Create a Repository resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: RepositoryArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Repository resources.
 */
export interface RepositoryState {
    /**
     * Full ARN of the repository.
     */
    arn?: pulumi.Input<string>;
    /**
     * Encryption configuration for the repository. See below for schema.
     */
    encryptionConfigurations?: pulumi.Input<pulumi.Input<inputs.ecr.RepositoryEncryptionConfiguration>[]>;
    /**
     * If `true`, will delete the repository even if it contains images.
     * Defaults to `false`.
     */
    forceDelete?: pulumi.Input<boolean>;
    /**
     * Configuration block that defines image scanning configuration for the repository. By default, image scanning must be manually triggered. See the [ECR User Guide](https://docs.aws.amazon.com/AmazonECR/latest/userguide/image-scanning.html) for more information about image scanning.
     */
    imageScanningConfiguration?: pulumi.Input<inputs.ecr.RepositoryImageScanningConfiguration>;
    /**
     * The tag mutability setting for the repository. Must be one of: `MUTABLE`, `IMMUTABLE`, `IMMUTABLE_WITH_EXCLUSION`, or `MUTABLE_WITH_EXCLUSION`. Defaults to `MUTABLE`.
     */
    imageTagMutability?: pulumi.Input<string>;
    /**
     * Configuration block that defines filters to specify which image tags can override the default tag mutability setting. Only applicable when `imageTagMutability` is set to `IMMUTABLE_WITH_EXCLUSION` or `MUTABLE_WITH_EXCLUSION`. See below for schema.
     */
    imageTagMutabilityExclusionFilters?: pulumi.Input<pulumi.Input<inputs.ecr.RepositoryImageTagMutabilityExclusionFilter>[]>;
    /**
     * Name of the repository.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The registry ID where the repository was created.
     */
    registryId?: pulumi.Input<string>;
    /**
     * The URL of the repository (in the form `aws_account_id.dkr.ecr.region.amazonaws.com/repositoryName`).
     */
    repositoryUrl?: pulumi.Input<string>;
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
 * The set of arguments for constructing a Repository resource.
 */
export interface RepositoryArgs {
    /**
     * Encryption configuration for the repository. See below for schema.
     */
    encryptionConfigurations?: pulumi.Input<pulumi.Input<inputs.ecr.RepositoryEncryptionConfiguration>[]>;
    /**
     * If `true`, will delete the repository even if it contains images.
     * Defaults to `false`.
     */
    forceDelete?: pulumi.Input<boolean>;
    /**
     * Configuration block that defines image scanning configuration for the repository. By default, image scanning must be manually triggered. See the [ECR User Guide](https://docs.aws.amazon.com/AmazonECR/latest/userguide/image-scanning.html) for more information about image scanning.
     */
    imageScanningConfiguration?: pulumi.Input<inputs.ecr.RepositoryImageScanningConfiguration>;
    /**
     * The tag mutability setting for the repository. Must be one of: `MUTABLE`, `IMMUTABLE`, `IMMUTABLE_WITH_EXCLUSION`, or `MUTABLE_WITH_EXCLUSION`. Defaults to `MUTABLE`.
     */
    imageTagMutability?: pulumi.Input<string>;
    /**
     * Configuration block that defines filters to specify which image tags can override the default tag mutability setting. Only applicable when `imageTagMutability` is set to `IMMUTABLE_WITH_EXCLUSION` or `MUTABLE_WITH_EXCLUSION`. See below for schema.
     */
    imageTagMutabilityExclusionFilters?: pulumi.Input<pulumi.Input<inputs.ecr.RepositoryImageTagMutabilityExclusionFilter>[]>;
    /**
     * Name of the repository.
     */
    name?: pulumi.Input<string>;
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
