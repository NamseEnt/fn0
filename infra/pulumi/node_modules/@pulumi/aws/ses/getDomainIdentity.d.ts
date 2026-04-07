import * as pulumi from "@pulumi/pulumi";
/**
 * Retrieve the SES domain identity
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ses.getDomainIdentity({
 *     domain: "example.com",
 * });
 * ```
 */
export declare function getDomainIdentity(args: GetDomainIdentityArgs, opts?: pulumi.InvokeOptions): Promise<GetDomainIdentityResult>;
/**
 * A collection of arguments for invoking getDomainIdentity.
 */
export interface GetDomainIdentityArgs {
    /**
     * Name of the domain
     */
    domain: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getDomainIdentity.
 */
export interface GetDomainIdentityResult {
    /**
     * ARN of the domain identity.
     */
    readonly arn: string;
    /**
     * Name of the domain
     */
    readonly domain: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
    /**
     * Code which when added to the domain as a TXT record will signal to SES that the owner of the domain has authorized SES to act on their behalf.
     */
    readonly verificationToken: string;
}
/**
 * Retrieve the SES domain identity
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ses.getDomainIdentity({
 *     domain: "example.com",
 * });
 * ```
 */
export declare function getDomainIdentityOutput(args: GetDomainIdentityOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetDomainIdentityResult>;
/**
 * A collection of arguments for invoking getDomainIdentity.
 */
export interface GetDomainIdentityOutputArgs {
    /**
     * Name of the domain
     */
    domain: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
