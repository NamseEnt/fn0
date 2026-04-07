import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Returns a list of Amazon Bedrock custom models.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.bedrock.getCustomModels({});
 * ```
 */
export declare function getCustomModels(args?: GetCustomModelsArgs, opts?: pulumi.InvokeOptions): Promise<GetCustomModelsResult>;
/**
 * A collection of arguments for invoking getCustomModels.
 */
export interface GetCustomModelsArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getCustomModels.
 */
export interface GetCustomModelsResult {
    readonly id: string;
    /**
     * Model summaries.
     */
    readonly modelSummaries: outputs.bedrock.GetCustomModelsModelSummary[];
    readonly region: string;
}
/**
 * Returns a list of Amazon Bedrock custom models.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.bedrock.getCustomModels({});
 * ```
 */
export declare function getCustomModelsOutput(args?: GetCustomModelsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetCustomModelsResult>;
/**
 * A collection of arguments for invoking getCustomModels.
 */
export interface GetCustomModelsOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
