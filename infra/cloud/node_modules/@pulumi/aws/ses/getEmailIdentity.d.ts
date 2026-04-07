import * as pulumi from "@pulumi/pulumi";
/**
 * Retrieve the active SES email identity
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ses.getEmailIdentity({
 *     email: "awesome@example.com",
 * });
 * ```
 */
export declare function getEmailIdentity(args: GetEmailIdentityArgs, opts?: pulumi.InvokeOptions): Promise<GetEmailIdentityResult>;
/**
 * A collection of arguments for invoking getEmailIdentity.
 */
export interface GetEmailIdentityArgs {
    /**
     * Email identity.
     */
    email: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getEmailIdentity.
 */
export interface GetEmailIdentityResult {
    /**
     * The ARN of the email identity.
     */
    readonly arn: string;
    /**
     * Email identity.
     */
    readonly email: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly region: string;
}
/**
 * Retrieve the active SES email identity
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.ses.getEmailIdentity({
 *     email: "awesome@example.com",
 * });
 * ```
 */
export declare function getEmailIdentityOutput(args: GetEmailIdentityOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetEmailIdentityResult>;
/**
 * A collection of arguments for invoking getEmailIdentity.
 */
export interface GetEmailIdentityOutputArgs {
    /**
     * Email identity.
     */
    email: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
