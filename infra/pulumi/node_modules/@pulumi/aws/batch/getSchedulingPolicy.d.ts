import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * The Batch Scheduling Policy data source allows access to details of a specific Scheduling Policy within AWS Batch.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.batch.getSchedulingPolicy({
 *     arn: "arn:aws:batch:us-east-1:012345678910:scheduling-policy/example",
 * });
 * ```
 */
export declare function getSchedulingPolicy(args: GetSchedulingPolicyArgs, opts?: pulumi.InvokeOptions): Promise<GetSchedulingPolicyResult>;
/**
 * A collection of arguments for invoking getSchedulingPolicy.
 */
export interface GetSchedulingPolicyArgs {
    /**
     * ARN of the scheduling policy.
     */
    arn: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Key-value map of resource tags
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getSchedulingPolicy.
 */
export interface GetSchedulingPolicyResult {
    readonly arn: string;
    readonly fairSharePolicies: outputs.batch.GetSchedulingPolicyFairSharePolicy[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Name of the scheduling policy.
     */
    readonly name: string;
    readonly region: string;
    /**
     * Key-value map of resource tags
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * The Batch Scheduling Policy data source allows access to details of a specific Scheduling Policy within AWS Batch.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.batch.getSchedulingPolicy({
 *     arn: "arn:aws:batch:us-east-1:012345678910:scheduling-policy/example",
 * });
 * ```
 */
export declare function getSchedulingPolicyOutput(args: GetSchedulingPolicyOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetSchedulingPolicyResult>;
/**
 * A collection of arguments for invoking getSchedulingPolicy.
 */
export interface GetSchedulingPolicyOutputArgs {
    /**
     * ARN of the scheduling policy.
     */
    arn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
