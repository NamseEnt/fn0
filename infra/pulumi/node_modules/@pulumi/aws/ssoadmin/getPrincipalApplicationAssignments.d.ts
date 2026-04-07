import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Data source for viewing AWS SSO Admin Principal Application Assignments.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getPrincipalApplicationAssignments({
 *     instanceArn: test.arns[0],
 *     principalId: testAwsIdentitystoreUser.userId,
 *     principalType: "USER",
 * });
 * ```
 */
export declare function getPrincipalApplicationAssignments(args: GetPrincipalApplicationAssignmentsArgs, opts?: pulumi.InvokeOptions): Promise<GetPrincipalApplicationAssignmentsResult>;
/**
 * A collection of arguments for invoking getPrincipalApplicationAssignments.
 */
export interface GetPrincipalApplicationAssignmentsArgs {
    /**
     * List of principals assigned to the application. See the `applicationAssignments` attribute reference below.
     */
    applicationAssignments?: inputs.ssoadmin.GetPrincipalApplicationAssignmentsApplicationAssignment[];
    /**
     * ARN of the instance of IAM Identity Center.
     */
    instanceArn: string;
    /**
     * An identifier for an object in IAM Identity Center, such as a user or group.
     */
    principalId: string;
    /**
     * Entity type for which the assignment will be created. Valid values are `USER` or `GROUP`.
     */
    principalType: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getPrincipalApplicationAssignments.
 */
export interface GetPrincipalApplicationAssignmentsResult {
    /**
     * List of principals assigned to the application. See the `applicationAssignments` attribute reference below.
     */
    readonly applicationAssignments?: outputs.ssoadmin.GetPrincipalApplicationAssignmentsApplicationAssignment[];
    readonly id: string;
    readonly instanceArn: string;
    /**
     * An identifier for an object in IAM Identity Center, such as a user or group.
     */
    readonly principalId: string;
    /**
     * Entity type for which the assignment will be created. Valid values are `USER` or `GROUP`.
     */
    readonly principalType: string;
    readonly region: string;
}
/**
 * Data source for viewing AWS SSO Admin Principal Application Assignments.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ssoadmin.getPrincipalApplicationAssignments({
 *     instanceArn: test.arns[0],
 *     principalId: testAwsIdentitystoreUser.userId,
 *     principalType: "USER",
 * });
 * ```
 */
export declare function getPrincipalApplicationAssignmentsOutput(args: GetPrincipalApplicationAssignmentsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetPrincipalApplicationAssignmentsResult>;
/**
 * A collection of arguments for invoking getPrincipalApplicationAssignments.
 */
export interface GetPrincipalApplicationAssignmentsOutputArgs {
    /**
     * List of principals assigned to the application. See the `applicationAssignments` attribute reference below.
     */
    applicationAssignments?: pulumi.Input<pulumi.Input<inputs.ssoadmin.GetPrincipalApplicationAssignmentsApplicationAssignmentArgs>[]>;
    /**
     * ARN of the instance of IAM Identity Center.
     */
    instanceArn: pulumi.Input<string>;
    /**
     * An identifier for an object in IAM Identity Center, such as a user or group.
     */
    principalId: pulumi.Input<string>;
    /**
     * Entity type for which the assignment will be created. Valid values are `USER` or `GROUP`.
     */
    principalType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
