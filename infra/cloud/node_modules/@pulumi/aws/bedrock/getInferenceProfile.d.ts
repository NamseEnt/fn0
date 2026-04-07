import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS Bedrock Inference Profile.
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
 * const testGetInferenceProfile = test.then(test => aws.bedrock.getInferenceProfile({
 *     inferenceProfileId: test.inferenceProfileSummaries?.[0]?.inferenceProfileId,
 * }));
 * ```
 */
export declare function getInferenceProfile(args: GetInferenceProfileArgs, opts?: pulumi.InvokeOptions): Promise<GetInferenceProfileResult>;
/**
 * A collection of arguments for invoking getInferenceProfile.
 */
export interface GetInferenceProfileArgs {
    /**
     * Inference Profile identifier.
     */
    inferenceProfileId: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getInferenceProfile.
 */
export interface GetInferenceProfileResult {
    /**
     * The time at which the inference profile was created.
     */
    readonly createdAt: string;
    /**
     * The description of the inference profile.
     */
    readonly description: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * The Amazon Resource Name (ARN) of the inference profile.
     */
    readonly inferenceProfileArn: string;
    readonly inferenceProfileId: string;
    /**
     * The unique identifier of the inference profile.
     */
    readonly inferenceProfileName: string;
    /**
     * A list of information about each model in the inference profile. See `models`.
     */
    readonly models: outputs.bedrock.GetInferenceProfileModel[];
    readonly region: string;
    /**
     * The status of the inference profile. `ACTIVE` means that the inference profile is available to use.
     */
    readonly status: string;
    /**
     * The type of the inference profile. `SYSTEM_DEFINED` means that the inference profile is defined by Amazon Bedrock. `APPLICATION` means that the inference profile is defined by the user.
     */
    readonly type: string;
    /**
     * The time at which the inference profile was last updated.
     */
    readonly updatedAt: string;
}
/**
 * Data source for managing an AWS Bedrock Inference Profile.
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
 * const testGetInferenceProfile = test.then(test => aws.bedrock.getInferenceProfile({
 *     inferenceProfileId: test.inferenceProfileSummaries?.[0]?.inferenceProfileId,
 * }));
 * ```
 */
export declare function getInferenceProfileOutput(args: GetInferenceProfileOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetInferenceProfileResult>;
/**
 * A collection of arguments for invoking getInferenceProfile.
 */
export interface GetInferenceProfileOutputArgs {
    /**
     * Inference Profile identifier.
     */
    inferenceProfileId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
