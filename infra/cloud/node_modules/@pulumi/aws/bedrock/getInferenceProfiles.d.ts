import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing AWS Bedrock Inference Profiles.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.bedrock.getInferenceProfiles({});
 * ```
 *
 * ### Filter by Type
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.bedrock.getInferenceProfiles({
 *     type: "APPLICATION",
 * });
 * ```
 */
export declare function getInferenceProfiles(args?: GetInferenceProfilesArgs, opts?: pulumi.InvokeOptions): Promise<GetInferenceProfilesResult>;
/**
 * A collection of arguments for invoking getInferenceProfiles.
 */
export interface GetInferenceProfilesArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Filters for inference profiles that match the type you specify. Valid values are: `SYSTEM_DEFINED`, `APPLICATION`.
     */
    type?: string;
}
/**
 * A collection of values returned by getInferenceProfiles.
 */
export interface GetInferenceProfilesResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * List of inference profile summary objects. See `inferenceProfileSummaries`.
     */
    readonly inferenceProfileSummaries: outputs.bedrock.GetInferenceProfilesInferenceProfileSummary[];
    readonly region: string;
    /**
     * Type of the inference profile. `SYSTEM_DEFINED` means that the inference profile is defined by Amazon Bedrock. `APPLICATION` means the inference profile was created by a user.
     */
    readonly type?: string;
}
/**
 * Data source for managing AWS Bedrock Inference Profiles.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.bedrock.getInferenceProfiles({});
 * ```
 *
 * ### Filter by Type
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.bedrock.getInferenceProfiles({
 *     type: "APPLICATION",
 * });
 * ```
 */
export declare function getInferenceProfilesOutput(args?: GetInferenceProfilesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetInferenceProfilesResult>;
/**
 * A collection of arguments for invoking getInferenceProfiles.
 */
export interface GetInferenceProfilesOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Filters for inference profiles that match the type you specify. Valid values are: `SYSTEM_DEFINED`, `APPLICATION`.
     */
    type?: pulumi.Input<string>;
}
