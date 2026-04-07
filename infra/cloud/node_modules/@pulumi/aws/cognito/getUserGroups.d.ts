import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing AWS Cognito IDP (Identity Provider) User Groups.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cognito.getUserGroups({
 *     userPoolId: "us-west-2_aaaaaaaaa",
 * });
 * ```
 */
export declare function getUserGroups(args: GetUserGroupsArgs, opts?: pulumi.InvokeOptions): Promise<GetUserGroupsResult>;
/**
 * A collection of arguments for invoking getUserGroups.
 */
export interface GetUserGroupsArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * User pool the client belongs to.
     */
    userPoolId: string;
}
/**
 * A collection of values returned by getUserGroups.
 */
export interface GetUserGroupsResult {
    /**
     * List of groups. See `groups` below.
     */
    readonly groups: outputs.cognito.GetUserGroupsGroup[];
    /**
     * User pool identifier.
     */
    readonly id: string;
    readonly region: string;
    readonly userPoolId: string;
}
/**
 * Data source for managing AWS Cognito IDP (Identity Provider) User Groups.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cognito.getUserGroups({
 *     userPoolId: "us-west-2_aaaaaaaaa",
 * });
 * ```
 */
export declare function getUserGroupsOutput(args: GetUserGroupsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetUserGroupsResult>;
/**
 * A collection of arguments for invoking getUserGroups.
 */
export interface GetUserGroupsOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * User pool the client belongs to.
     */
    userPoolId: pulumi.Input<string>;
}
