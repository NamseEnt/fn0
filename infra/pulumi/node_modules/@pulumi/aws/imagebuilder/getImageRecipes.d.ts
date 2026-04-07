import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Use this data source to get the ARNs and names of Image Builder Image Recipes matching the specified criteria.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.imagebuilder.getImageRecipes({
 *     owner: "Self",
 *     filters: [{
 *         name: "platform",
 *         values: ["Linux"],
 *     }],
 * });
 * ```
 */
export declare function getImageRecipes(args?: GetImageRecipesArgs, opts?: pulumi.InvokeOptions): Promise<GetImageRecipesResult>;
/**
 * A collection of arguments for invoking getImageRecipes.
 */
export interface GetImageRecipesArgs {
    /**
     * Configuration block(s) for filtering. Detailed below.
     */
    filters?: inputs.imagebuilder.GetImageRecipesFilter[];
    /**
     * Owner of the image recipes. Valid values are `Self`, `Shared`, `Amazon` and `ThirdParty`. Defaults to `Self`.
     */
    owner?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getImageRecipes.
 */
export interface GetImageRecipesResult {
    /**
     * Set of ARNs of the matched Image Builder Image Recipes.
     */
    readonly arns: string[];
    readonly filters?: outputs.imagebuilder.GetImageRecipesFilter[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Set of names of the matched Image Builder Image Recipes.
     */
    readonly names: string[];
    readonly owner?: string;
    readonly region: string;
}
/**
 * Use this data source to get the ARNs and names of Image Builder Image Recipes matching the specified criteria.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.imagebuilder.getImageRecipes({
 *     owner: "Self",
 *     filters: [{
 *         name: "platform",
 *         values: ["Linux"],
 *     }],
 * });
 * ```
 */
export declare function getImageRecipesOutput(args?: GetImageRecipesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetImageRecipesResult>;
/**
 * A collection of arguments for invoking getImageRecipes.
 */
export interface GetImageRecipesOutputArgs {
    /**
     * Configuration block(s) for filtering. Detailed below.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.imagebuilder.GetImageRecipesFilterArgs>[]>;
    /**
     * Owner of the image recipes. Valid values are `Self`, `Shared`, `Amazon` and `ThirdParty`. Defaults to `Self`.
     */
    owner?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
