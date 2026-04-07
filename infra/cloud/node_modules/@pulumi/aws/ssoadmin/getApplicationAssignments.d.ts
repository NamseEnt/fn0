import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing AWS SSO Admin Application Assignments.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getApplicationAssignments({
 *     applicationArn: exampleAwsSsoadminApplication.arn,
 * });
 * ```
 */
export declare function getApplicationAssignments(args: GetApplicationAssignmentsArgs, opts?: pulumi.InvokeOptions): Promise<GetApplicationAssignmentsResult>;
/**
 * A collection of arguments for invoking getApplicationAssignments.
 */
export interface GetApplicationAssignmentsArgs {
    /**
     * ARN of the application.
     */
    applicationArn: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getApplicationAssignments.
 */
export interface GetApplicationAssignmentsResult {
    /**
     * ARN of the application.
     */
    readonly applicationArn: string;
    /**
     * List of principals assigned to the application. See the `applicationAssignments` attribute reference below.
     */
    readonly applicationAssignments: outputs.ssoadmin.GetApplicationAssignmentsApplicationAssignment[];
    readonly id: string;
    readonly region: string;
}
/**
 * Data source for managing AWS SSO Admin Application Assignments.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getApplicationAssignments({
 *     applicationArn: exampleAwsSsoadminApplication.arn,
 * });
 * ```
 */
export declare function getApplicationAssignmentsOutput(args: GetApplicationAssignmentsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetApplicationAssignmentsResult>;
/**
 * A collection of arguments for invoking getApplicationAssignments.
 */
export interface GetApplicationAssignmentsOutputArgs {
    /**
     * ARN of the application.
     */
    applicationArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
