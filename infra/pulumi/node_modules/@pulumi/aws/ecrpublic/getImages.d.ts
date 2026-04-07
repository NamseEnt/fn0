import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * The ECR Public Images data source allows the list of images in a specified public repository to be retrieved.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = aws.ecrpublic.getImages({
 *     repositoryName: "my-public-repository",
 * });
 * export const imageDigests = example.then(example => .filter(img => img.digest != null).map(img => (img.digest)));
 * export const imageTags = example.then(example => std.flatten({
 *     input: .map(img => (img.tags)),
 * })).then(invoke => std.distinct({
 *     input: invoke.result,
 * })).then(invoke => invoke.result);
 * ```
 */
export declare function getImages(args: GetImagesArgs, opts?: pulumi.InvokeOptions): Promise<GetImagesResult>;
/**
 * A collection of arguments for invoking getImages.
 */
export interface GetImagesArgs {
    /**
     * One or more image ID filters. Each image ID can use either a tag or digest (or both). Each object has the following attributes:
     */
    imageIds?: inputs.ecrpublic.GetImagesImageId[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * AWS account ID associated with the public registry that contains the repository. If not specified, the default public registry is assumed.
     */
    registryId?: string;
    /**
     * Name of the public repository.
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
    readonly imageIds?: outputs.ecrpublic.GetImagesImageId[];
    /**
     * List of images returned. Each image contains:
     */
    readonly images: outputs.ecrpublic.GetImagesImage[];
    readonly region: string;
    /**
     * AWS account ID associated with the public registry.
     */
    readonly registryId?: string;
    /**
     * Name of the repository.
     */
    readonly repositoryName: string;
}
/**
 * The ECR Public Images data source allows the list of images in a specified public repository to be retrieved.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = aws.ecrpublic.getImages({
 *     repositoryName: "my-public-repository",
 * });
 * export const imageDigests = example.then(example => .filter(img => img.digest != null).map(img => (img.digest)));
 * export const imageTags = example.then(example => std.flatten({
 *     input: .map(img => (img.tags)),
 * })).then(invoke => std.distinct({
 *     input: invoke.result,
 * })).then(invoke => invoke.result);
 * ```
 */
export declare function getImagesOutput(args: GetImagesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetImagesResult>;
/**
 * A collection of arguments for invoking getImages.
 */
export interface GetImagesOutputArgs {
    /**
     * One or more image ID filters. Each image ID can use either a tag or digest (or both). Each object has the following attributes:
     */
    imageIds?: pulumi.Input<pulumi.Input<inputs.ecrpublic.GetImagesImageIdArgs>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * AWS account ID associated with the public registry that contains the repository. If not specified, the default public registry is assumed.
     */
    registryId?: pulumi.Input<string>;
    /**
     * Name of the public repository.
     */
    repositoryName: pulumi.Input<string>;
}
