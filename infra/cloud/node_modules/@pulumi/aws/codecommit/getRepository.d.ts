import * as pulumi from "@pulumi/pulumi";
/**
 * The CodeCommit Repository data source allows the ARN, Repository ID, Repository URL for HTTP and Repository URL for SSH to be retrieved for an CodeCommit repository.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.codecommit.getRepository({
 *     repositoryName: "MyTestRepository",
 * });
 * ```
 */
export declare function getRepository(args: GetRepositoryArgs, opts?: pulumi.InvokeOptions): Promise<GetRepositoryResult>;
/**
 * A collection of arguments for invoking getRepository.
 */
export interface GetRepositoryArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Name for the repository. This needs to be less than 100 characters.
     */
    repositoryName: string;
}
/**
 * A collection of values returned by getRepository.
 */
export interface GetRepositoryResult {
    /**
     * ARN of the repository.
     */
    readonly arn: string;
    /**
     * URL to use for cloning the repository over HTTPS.
     */
    readonly cloneUrlHttp: string;
    /**
     * URL to use for cloning the repository over SSH.
     */
    readonly cloneUrlSsh: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * The ID of the encryption key.
     */
    readonly kmsKeyId: string;
    readonly region: string;
    /**
     * ID of the repository.
     */
    readonly repositoryId: string;
    readonly repositoryName: string;
}
/**
 * The CodeCommit Repository data source allows the ARN, Repository ID, Repository URL for HTTP and Repository URL for SSH to be retrieved for an CodeCommit repository.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.codecommit.getRepository({
 *     repositoryName: "MyTestRepository",
 * });
 * ```
 */
export declare function getRepositoryOutput(args: GetRepositoryOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetRepositoryResult>;
/**
 * A collection of arguments for invoking getRepository.
 */
export interface GetRepositoryOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name for the repository. This needs to be less than 100 characters.
     */
    repositoryName: pulumi.Input<string>;
}
