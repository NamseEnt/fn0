import * as pulumi from "@pulumi/pulumi";
/**
 * Use this data source to get the signing certificate for a Cognito IdP user pool.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const sc = aws.cognito.getUserPoolSigningCertificate({
 *     userPoolId: myPool.id,
 * });
 * ```
 */
export declare function getUserPoolSigningCertificate(args: GetUserPoolSigningCertificateArgs, opts?: pulumi.InvokeOptions): Promise<GetUserPoolSigningCertificateResult>;
/**
 * A collection of arguments for invoking getUserPoolSigningCertificate.
 */
export interface GetUserPoolSigningCertificateArgs {
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
 * A collection of values returned by getUserPoolSigningCertificate.
 */
export interface GetUserPoolSigningCertificateResult {
    /**
     * Certificate string
     */
    readonly certificate: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    readonly userPoolId: string;
}
/**
 * Use this data source to get the signing certificate for a Cognito IdP user pool.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const sc = aws.cognito.getUserPoolSigningCertificate({
 *     userPoolId: myPool.id,
 * });
 * ```
 */
export declare function getUserPoolSigningCertificateOutput(args: GetUserPoolSigningCertificateOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetUserPoolSigningCertificateResult>;
/**
 * A collection of arguments for invoking getUserPoolSigningCertificate.
 */
export interface GetUserPoolSigningCertificateOutputArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Cognito user pool ID.
     */
    userPoolId: pulumi.Input<string>;
}
