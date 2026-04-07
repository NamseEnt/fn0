import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing AWS Bedrock Foundation Models.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.bedrockfoundation.getModels({});
 * ```
 *
 * ### Filter by Inference Type
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.bedrockfoundation.getModels({
 *     byInferenceType: "ON_DEMAND",
 * });
 * ```
 */
export declare function getModels(args?: GetModelsArgs, opts?: pulumi.InvokeOptions): Promise<GetModelsResult>;
/**
 * A collection of arguments for invoking getModels.
 */
export interface GetModelsArgs {
    /**
     * Customization type to filter on. Valid values are `FINE_TUNING`.
     */
    byCustomizationType?: string;
    /**
     * Inference type to filter on. Valid values are `ON_DEMAND` and `PROVISIONED`.
     */
    byInferenceType?: string;
    /**
     * Output modality to filter on. Valid values are `TEXT`, `IMAGE`, and `EMBEDDING`.
     */
    byOutputModality?: string;
    /**
     * Model provider to filter on.
     */
    byProvider?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getModels.
 */
export interface GetModelsResult {
    readonly byCustomizationType?: string;
    readonly byInferenceType?: string;
    readonly byOutputModality?: string;
    readonly byProvider?: string;
    /**
     * AWS region.
     */
    readonly id: string;
    /**
     * List of model summary objects. See `modelSummaries`.
     */
    readonly modelSummaries: outputs.bedrockfoundation.GetModelsModelSummary[];
    readonly region: string;
}
/**
 * Data source for managing AWS Bedrock Foundation Models.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.bedrockfoundation.getModels({});
 * ```
 *
 * ### Filter by Inference Type
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.bedrockfoundation.getModels({
 *     byInferenceType: "ON_DEMAND",
 * });
 * ```
 */
export declare function getModelsOutput(args?: GetModelsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetModelsResult>;
/**
 * A collection of arguments for invoking getModels.
 */
export interface GetModelsOutputArgs {
    /**
     * Customization type to filter on. Valid values are `FINE_TUNING`.
     */
    byCustomizationType?: pulumi.Input<string>;
    /**
     * Inference type to filter on. Valid values are `ON_DEMAND` and `PROVISIONED`.
     */
    byInferenceType?: pulumi.Input<string>;
    /**
     * Output modality to filter on. Valid values are `TEXT`, `IMAGE`, and `EMBEDDING`.
     */
    byOutputModality?: pulumi.Input<string>;
    /**
     * Model provider to filter on.
     */
    byProvider?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
