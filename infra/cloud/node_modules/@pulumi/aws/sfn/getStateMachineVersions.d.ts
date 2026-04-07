import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing an AWS SFN (Step Functions) State Machine Versions.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.sfn.getStateMachineVersions({
 *     statemachineArn: testAwsSfnStateMachine.arn,
 * });
 * ```
 */
export declare function getStateMachineVersions(args: GetStateMachineVersionsArgs, opts?: pulumi.InvokeOptions): Promise<GetStateMachineVersionsResult>;
/**
 * A collection of arguments for invoking getStateMachineVersions.
 */
export interface GetStateMachineVersionsArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * ARN of the State Machine.
     */
    statemachineArn: string;
}
/**
 * A collection of values returned by getStateMachineVersions.
 */
export interface GetStateMachineVersionsResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    readonly statemachineArn: string;
    /**
     * ARN List identifying the statemachine versions.
     */
    readonly statemachineVersions: string[];
}
/**
 * Data source for managing an AWS SFN (Step Functions) State Machine Versions.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.sfn.getStateMachineVersions({
 *     statemachineArn: testAwsSfnStateMachine.arn,
 * });
 * ```
 */
export declare function getStateMachineVersionsOutput(args: GetStateMachineVersionsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetStateMachineVersionsResult>;
/**
 * A collection of arguments for invoking getStateMachineVersions.
 */
export interface GetStateMachineVersionsOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the State Machine.
     */
    statemachineArn: pulumi.Input<string>;
}
