import * as pulumi from "@pulumi/pulumi";
/**
 * The Public ECR Authorization Token data source allows the authorization token, token expiration date, user name, and password to be retrieved for a Public ECR repository.
 *
 * > **NOTE:** This data source can only be used in the `us-east-1` region.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const token = aws.ecrpublic.getAuthorizationToken({});
 * ```
 */
export declare function getAuthorizationToken(args?: GetAuthorizationTokenArgs, opts?: pulumi.InvokeOptions): Promise<GetAuthorizationTokenResult>;
/**
 * A collection of arguments for invoking getAuthorizationToken.
 */
export interface GetAuthorizationTokenArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getAuthorizationToken.
 */
export interface GetAuthorizationTokenResult {
    /**
     * Temporary IAM authentication credentials to access the ECR repository encoded in base64 in the form of `user_name:password`.
     */
    readonly authorizationToken: string;
    /**
     * Time in UTC RFC3339 format when the authorization token expires.
     */
    readonly expiresAt: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * Password decoded from the authorization token.
     */
    readonly password: string;
    readonly region: string;
    /**
     * User name decoded from the authorization token.
     */
    readonly userName: string;
}
/**
 * The Public ECR Authorization Token data source allows the authorization token, token expiration date, user name, and password to be retrieved for a Public ECR repository.
 *
 * > **NOTE:** This data source can only be used in the `us-east-1` region.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const token = aws.ecrpublic.getAuthorizationToken({});
 * ```
 */
export declare function getAuthorizationTokenOutput(args?: GetAuthorizationTokenOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetAuthorizationTokenResult>;
/**
 * A collection of arguments for invoking getAuthorizationToken.
 */
export interface GetAuthorizationTokenOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
