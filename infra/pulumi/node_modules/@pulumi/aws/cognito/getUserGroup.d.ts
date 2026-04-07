import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing an AWS Cognito IDP (Identity Provider) User Group.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cognito.getUserGroup({
 *     userPoolId: "us-west-2_aaaaaaaaa",
 *     name: "example",
 * });
 * ```
 */
export declare function getUserGroup(args: GetUserGroupArgs, opts?: pulumi.InvokeOptions): Promise<GetUserGroupResult>;
/**
 * A collection of arguments for invoking getUserGroup.
 */
export interface GetUserGroupArgs {
    /**
     * Name of the user group.
     */
    name: string;
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
 * A collection of values returned by getUserGroup.
 */
export interface GetUserGroupResult {
    /**
     * Description of the user group.
     */
    readonly description: string;
    /**
     * A comma-delimited string concatenating `name` and `userPoolId`.
     */
    readonly id: string;
    readonly name: string;
    /**
     * Precedence of the user group.
     */
    readonly precedence: number;
    readonly region: string;
    /**
     * ARN of the IAM role to be associated with the user group.
     */
    readonly roleArn: string;
    readonly userPoolId: string;
}
/**
 * Data source for managing an AWS Cognito IDP (Identity Provider) User Group.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.cognito.getUserGroup({
 *     userPoolId: "us-west-2_aaaaaaaaa",
 *     name: "example",
 * });
 * ```
 */
export declare function getUserGroupOutput(args: GetUserGroupOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetUserGroupResult>;
/**
 * A collection of arguments for invoking getUserGroup.
 */
export interface GetUserGroupOutputArgs {
    /**
     * Name of the user group.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * User pool the client belongs to.
     */
    userPoolId: pulumi.Input<string>;
}
