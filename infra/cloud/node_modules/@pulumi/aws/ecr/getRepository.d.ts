import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * The ECR Repository data source allows the ARN, Repository URI and Registry ID to be retrieved for an ECR repository.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const service = aws.ecr.getRepository({
 *     name: "ecr-repository",
 * });
 * ```
 */
export declare function getRepository(args: GetRepositoryArgs, opts?: pulumi.InvokeOptions): Promise<GetRepositoryResult>;
/**
 * A collection of arguments for invoking getRepository.
 */
export interface GetRepositoryArgs {
    /**
     * Name of the ECR Repository.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Registry ID where the repository was created.
     */
    registryId?: string;
    /**
     * Map of tags assigned to the resource.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getRepository.
 */
export interface GetRepositoryResult {
    /**
     * Full ARN of the repository.
     */
    readonly arn: string;
    /**
     * Encryption configuration for the repository. See Encryption Configuration below.
     */
    readonly encryptionConfigurations: outputs.ecr.GetRepositoryEncryptionConfiguration[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Configuration block that defines image scanning configuration for the repository. See Image Scanning Configuration below.
     */
    readonly imageScanningConfigurations: outputs.ecr.GetRepositoryImageScanningConfiguration[];
    /**
     * The tag mutability setting for the repository.
     */
    readonly imageTagMutability: string;
    /**
     * Block that defines filters to specify which image tags can override the default tag mutability setting.
     */
    readonly imageTagMutabilityExclusionFilters: outputs.ecr.GetRepositoryImageTagMutabilityExclusionFilter[];
    /**
     * List of image tags associated with the most recently pushed image in the repository.
     */
    readonly mostRecentImageTags: string[];
    readonly name: string;
    readonly region: string;
    readonly registryId: string;
    /**
     * URL of the repository (in the form `aws_account_id.dkr.ecr.region.amazonaws.com/repositoryName`).
     */
    readonly repositoryUrl: string;
    /**
     * Map of tags assigned to the resource.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * The ECR Repository data source allows the ARN, Repository URI and Registry ID to be retrieved for an ECR repository.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const service = aws.ecr.getRepository({
 *     name: "ecr-repository",
 * });
 * ```
 */
export declare function getRepositoryOutput(args: GetRepositoryOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetRepositoryResult>;
/**
 * A collection of arguments for invoking getRepository.
 */
export interface GetRepositoryOutputArgs {
    /**
     * Name of the ECR Repository.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Registry ID where the repository was created.
     */
    registryId?: pulumi.Input<string>;
    /**
     * Map of tags assigned to the resource.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
