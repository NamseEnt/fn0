import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS DevOps Guru Resource Collection.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.devopsguru.getResourceCollection({
 *     type: "AWS_SERVICE",
 * });
 * ```
 */
export declare function getResourceCollection(args: GetResourceCollectionArgs, opts?: pulumi.InvokeOptions): Promise<GetResourceCollectionResult>;
/**
 * A collection of arguments for invoking getResourceCollection.
 */
export interface GetResourceCollectionArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Type of AWS resource collection to create. Valid values are `AWS_CLOUD_FORMATION`, `AWS_SERVICE`, and `AWS_TAGS`.
     */
    type: string;
}
/**
 * A collection of values returned by getResourceCollection.
 */
export interface GetResourceCollectionResult {
    /**
     * A collection of AWS CloudFormation stacks. See `cloudformation` below for additional details.
     */
    readonly cloudformations: outputs.devopsguru.GetResourceCollectionCloudformation[];
    /**
     * Type of AWS resource collection to create (same value as `type`).
     */
    readonly id: string;
    readonly region: string;
    /**
     * AWS tags used to filter the resources in the resource collection. See `tags` below for additional details.
     */
    readonly tags: outputs.devopsguru.GetResourceCollectionTag[];
    readonly type: string;
}
/**
 * Data source for managing an AWS DevOps Guru Resource Collection.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.devopsguru.getResourceCollection({
 *     type: "AWS_SERVICE",
 * });
 * ```
 */
export declare function getResourceCollectionOutput(args: GetResourceCollectionOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetResourceCollectionResult>;
/**
 * A collection of arguments for invoking getResourceCollection.
 */
export interface GetResourceCollectionOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Type of AWS resource collection to create. Valid values are `AWS_CLOUD_FORMATION`, `AWS_SERVICE`, and `AWS_TAGS`.
     */
    type: pulumi.Input<string>;
}
