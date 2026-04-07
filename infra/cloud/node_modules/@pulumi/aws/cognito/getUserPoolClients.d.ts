import * as pulumi from "@pulumi/pulumi";
/**
 * Use this data source to get a list of Cognito user pools clients for a Cognito IdP user pool.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = aws.cognito.getUserPoolClients({
 *     userPoolId: mainAwsCognitoUserPool.id,
 * });
 * ```
 */
export declare function getUserPoolClients(args: GetUserPoolClientsArgs, opts?: pulumi.InvokeOptions): Promise<GetUserPoolClientsResult>;
/**
 * A collection of arguments for invoking getUserPoolClients.
 */
export interface GetUserPoolClientsArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Cognito user pool ID.
     */
    userPoolId: string;
}
/**
 * A collection of values returned by getUserPoolClients.
 */
export interface GetUserPoolClientsResult {
    /**
     * List of Cognito user pool client IDs.
     */
    readonly clientIds: string[];
    /**
     * List of Cognito user pool client names.
     */
    readonly clientNames: string[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    readonly userPoolId: string;
}
/**
 * Use this data source to get a list of Cognito user pools clients for a Cognito IdP user pool.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const main = aws.cognito.getUserPoolClients({
 *     userPoolId: mainAwsCognitoUserPool.id,
 * });
 * ```
 */
export declare function getUserPoolClientsOutput(args: GetUserPoolClientsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetUserPoolClientsResult>;
/**
 * A collection of arguments for invoking getUserPoolClients.
 */
export interface GetUserPoolClientsOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Cognito user pool ID.
     */
    userPoolId: pulumi.Input<string>;
}
