import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing an AWS SESv2 (Simple Email V2) Email Identity Mail From Attributes.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.sesv2.getEmailIdentity({
 *     emailIdentity: "example.com",
 * });
 * const exampleGetEmailIdentityMailFromAttributes = example.then(example => aws.sesv2.getEmailIdentityMailFromAttributes({
 *     emailIdentity: example.emailIdentity,
 * }));
 * ```
 */
export declare function getEmailIdentityMailFromAttributes(args: GetEmailIdentityMailFromAttributesArgs, opts?: pulumi.InvokeOptions): Promise<GetEmailIdentityMailFromAttributesResult>;
/**
 * A collection of arguments for invoking getEmailIdentityMailFromAttributes.
 */
export interface GetEmailIdentityMailFromAttributesArgs {
    /**
     * The name of the email identity.
     */
    emailIdentity: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getEmailIdentityMailFromAttributes.
 */
export interface GetEmailIdentityMailFromAttributesResult {
    /**
     * The action to take if the required MX record isn't found when you send an email. Valid values: `USE_DEFAULT_VALUE`, `REJECT_MESSAGE`.
     */
    readonly behaviorOnMxFailure: string;
    readonly emailIdentity: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * The custom MAIL FROM domain that you want the verified identity to use.
     */
    readonly mailFromDomain: string;
    readonly region: string;
}
/**
 * Data source for managing an AWS SESv2 (Simple Email V2) Email Identity Mail From Attributes.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.sesv2.getEmailIdentity({
 *     emailIdentity: "example.com",
 * });
 * const exampleGetEmailIdentityMailFromAttributes = example.then(example => aws.sesv2.getEmailIdentityMailFromAttributes({
 *     emailIdentity: example.emailIdentity,
 * }));
 * ```
 */
export declare function getEmailIdentityMailFromAttributesOutput(args: GetEmailIdentityMailFromAttributesOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetEmailIdentityMailFromAttributesResult>;
/**
 * A collection of arguments for invoking getEmailIdentityMailFromAttributes.
 */
export interface GetEmailIdentityMailFromAttributesOutputArgs {
    /**
     * The name of the email identity.
     */
    emailIdentity: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
