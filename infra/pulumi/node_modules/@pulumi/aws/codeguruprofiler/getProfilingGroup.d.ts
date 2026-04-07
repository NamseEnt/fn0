import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS CodeGuru Profiler Profiling Group.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.codeguruprofiler.getProfilingGroup({
 *     name: "example",
 * });
 * ```
 */
export declare function getProfilingGroup(args: GetProfilingGroupArgs, opts?: pulumi.InvokeOptions): Promise<GetProfilingGroupResult>;
/**
 * A collection of arguments for invoking getProfilingGroup.
 */
export interface GetProfilingGroupArgs {
    /**
     * The name of the profiling group.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getProfilingGroup.
 */
export interface GetProfilingGroupResult {
    /**
     * Profiling Group agent orchestration config
     */
    readonly agentOrchestrationConfigs: outputs.codeguruprofiler.GetProfilingGroupAgentOrchestrationConfig[];
    /**
     * ARN of the Profiling Group.
     */
    readonly arn: string;
    /**
     * The compute platform of the profiling group.
     */
    readonly computePlatform: string;
    /**
     * Timestamp when Profiling Group was created.
     */
    readonly createdAt: string;
    readonly id: string;
    readonly name: string;
    /**
     * The status of the Profiling Group.
     */
    readonly profilingStatuses: outputs.codeguruprofiler.GetProfilingGroupProfilingStatus[];
    readonly region: string;
    /**
     * Mapping of Key-Value tags for the resource.
     */
    readonly tags: {
        [key: string]: string;
    };
    /**
     * Timestamp when Profiling Group was updated.
     */
    readonly updatedAt: string;
}
/**
 * Data source for managing an AWS CodeGuru Profiler Profiling Group.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.codeguruprofiler.getProfilingGroup({
 *     name: "example",
 * });
 * ```
 */
export declare function getProfilingGroupOutput(args: GetProfilingGroupOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetProfilingGroupResult>;
/**
 * A collection of arguments for invoking getProfilingGroup.
 */
export interface GetProfilingGroupOutputArgs {
    /**
     * The name of the profiling group.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
