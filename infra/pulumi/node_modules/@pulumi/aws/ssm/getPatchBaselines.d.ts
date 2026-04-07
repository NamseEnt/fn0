import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Data source for retrieving AWS SSM (Systems Manager) Patch Baselines.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssm.getPatchBaselines({});
 * ```
 *
 * ### With Filters
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssm.getPatchBaselines({
 *     filters: [
 *         {
 *             key: "OWNER",
 *             values: ["AWS"],
 *         },
 *         {
 *             key: "OPERATING_SYSTEM",
 *             values: ["WINDOWS"],
 *         },
 *     ],
 * });
 * ```
 */
export declare function getPatchBaselines(args?: GetPatchBaselinesArgs, opts?: pulumi.InvokeOptions): Promise<GetPatchBaselinesResult>;
/**
 * A collection of arguments for invoking getPatchBaselines.
 */
export interface GetPatchBaselinesArgs {
    /**
     * Only return baseline identities where `defaultBaseline` is `true`.
     */
    defaultBaselines?: boolean;
    /**
     * Key-value pairs used to filter the results. See `filter` below.
     */
    filters?: inputs.ssm.GetPatchBaselinesFilter[];
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getPatchBaselines.
 */
export interface GetPatchBaselinesResult {
    /**
     * List of baseline identities. See `baselineIdentities` below.
     */
    readonly baselineIdentities: outputs.ssm.GetPatchBaselinesBaselineIdentity[];
    readonly defaultBaselines?: boolean;
    readonly filters?: outputs.ssm.GetPatchBaselinesFilter[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Data source for retrieving AWS SSM (Systems Manager) Patch Baselines.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssm.getPatchBaselines({});
 * ```
 *
 * ### With Filters
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssm.getPatchBaselines({
 *     filters: [
 *         {
 *             key: "OWNER",
 *             values: ["AWS"],
 *         },
 *         {
 *             key: "OPERATING_SYSTEM",
 *             values: ["WINDOWS"],
 *         },
 *     ],
 * });
 * ```
 */
export declare function getPatchBaselinesOutput(args?: GetPatchBaselinesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetPatchBaselinesResult>;
/**
 * A collection of arguments for invoking getPatchBaselines.
 */
export interface GetPatchBaselinesOutputArgs {
    /**
     * Only return baseline identities where `defaultBaseline` is `true`.
     */
    defaultBaselines?: pulumi.Input<boolean>;
    /**
     * Key-value pairs used to filter the results. See `filter` below.
     */
    filters?: pulumi.Input<pulumi.Input<inputs.ssm.GetPatchBaselinesFilterArgs>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
