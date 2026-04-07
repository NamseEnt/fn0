import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for providing information on AWS ECR (Elastic Container Registry) Repositories.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ecr.getRepositories({});
 * ```
 */
export declare function getRepositories(args?: GetRepositoriesArgs, opts?: pulumi.InvokeOptions): Promise<GetRepositoriesResult>;
/**
 * A collection of arguments for invoking getRepositories.
 */
export interface GetRepositoriesArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getRepositories.
 */
export interface GetRepositoriesResult {
    /**
     * AWS Region.
     */
    readonly id: string;
    /**
     * A list if AWS Elastic Container Registries for the region.
     */
    readonly names: string[];
    readonly region: string;
}
/**
 * Data source for providing information on AWS ECR (Elastic Container Registry) Repositories.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ecr.getRepositories({});
 * ```
 */
export declare function getRepositoriesOutput(args?: GetRepositoriesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetRepositoriesResult>;
/**
 * A collection of arguments for invoking getRepositories.
 */
export interface GetRepositoriesOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
