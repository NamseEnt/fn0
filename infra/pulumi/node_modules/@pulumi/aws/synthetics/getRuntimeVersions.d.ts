import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS CloudWatch Synthetics Runtime Versions.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.synthetics.getRuntimeVersions({});
 * ```
 */
export declare function getRuntimeVersions(args?: GetRuntimeVersionsArgs, opts?: pulumi.InvokeOptions): Promise<GetRuntimeVersionsResult>;
/**
 * A collection of arguments for invoking getRuntimeVersions.
 */
export interface GetRuntimeVersionsArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getRuntimeVersions.
 */
export interface GetRuntimeVersionsResult {
    /**
     * Name of the AWS region from which runtime versions are fetched.
     */
    readonly id: string;
    readonly region: string;
    /**
     * List of runtime versions. See `runtimeVersions` attribute reference.
     */
    readonly runtimeVersions: outputs.synthetics.GetRuntimeVersionsRuntimeVersion[];
}
/**
 * Data source for managing an AWS CloudWatch Synthetics Runtime Versions.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.synthetics.getRuntimeVersions({});
 * ```
 */
export declare function getRuntimeVersionsOutput(args?: GetRuntimeVersionsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetRuntimeVersionsResult>;
/**
 * A collection of arguments for invoking getRuntimeVersions.
 */
export interface GetRuntimeVersionsOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
