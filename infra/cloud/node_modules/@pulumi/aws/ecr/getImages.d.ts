import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * The ECR Images data source allows the list of images in a specified repository to be retrieved.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ecr.getImages({
 *     repositoryName: "my-repository",
 * });
 * export const imageDigests = example.then(example => .filter(img => img.imageDigest != null).map(img => (img.imageDigest)));
 * export const imageTags = example.then(example => .filter(img => img.imageTag != null).map(img => (img.imageTag)));
 * ```
 */
export declare function getImages(args: GetImagesArgs, opts?: pulumi.InvokeOptions): Promise<GetImagesResult>;
/**
 * A collection of arguments for invoking getImages.
 */
export interface GetImagesArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * ID of the Registry where the repository resides.
     */
    registryId?: string;
    /**
     * Name of the ECR Repository.
     */
    repositoryName: string;
}
/**
 * A collection of values returned by getImages.
 */
export interface GetImagesResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * List of image objects containing image digest and tags. Each object has the following attributes:
     */
    readonly imageIds: outputs.ecr.GetImagesImageId[];
    readonly region: string;
    readonly registryId?: string;
    readonly repositoryName: string;
}
/**
 * The ECR Images data source allows the list of images in a specified repository to be retrieved.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ecr.getImages({
 *     repositoryName: "my-repository",
 * });
 * export const imageDigests = example.then(example => .filter(img => img.imageDigest != null).map(img => (img.imageDigest)));
 * export const imageTags = example.then(example => .filter(img => img.imageTag != null).map(img => (img.imageTag)));
 * ```
 */
export declare function getImagesOutput(args: GetImagesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetImagesResult>;
/**
 * A collection of arguments for invoking getImages.
 */
export interface GetImagesOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the Registry where the repository resides.
     */
    registryId?: pulumi.Input<string>;
    /**
     * Name of the ECR Repository.
     */
    repositoryName: pulumi.Input<string>;
}
