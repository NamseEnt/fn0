import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Service Catalog Portfolio Share. Shares the specified portfolio with the specified account or organization node. You can share portfolios to an organization, an organizational unit, or a specific account.
 *
 * If the portfolio share with the specified account or organization node already exists, using this resource to re-create the share will have no effect and will not return an error. You can then use this resource to update the share.
 *
 * > **NOTE:** Shares to an organization node can only be created by the management account of an organization or by a delegated administrator. If a delegated admin is de-registered, they can no longer create portfolio shares.
 *
 * > **NOTE:** AWSOrganizationsAccess must be enabled in order to create a portfolio share to an organization node.
 *
 * > **NOTE:** You can't share a shared resource, including portfolios that contain a shared product.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicecatalog.PortfolioShare("example", {
 *     principalId: "012128675309",
 *     portfolioId: exampleAwsServicecatalogPortfolio.id,
 *     type: "ACCOUNT",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_servicecatalog_portfolio_share` using the portfolio share ID. For example:
 *
 * ```sh
 * $ pulumi import aws:servicecatalog/portfolioShare:PortfolioShare example port-12344321:ACCOUNT:123456789012
 * ```
 */
export declare class PortfolioShare extends pulumi.CustomResource {
    /**
     * Get an existing PortfolioShare resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PortfolioShareState, opts?: pulumi.CustomResourceOptions): PortfolioShare;
    /**
     * Returns true if the given object is an instance of PortfolioShare.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PortfolioShare;
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    readonly acceptLanguage: pulumi.Output<string | undefined>;
    /**
     * Whether the shared portfolio is imported by the recipient account. If the recipient is organizational, the share is automatically imported, and the field is always set to true.
     */
    readonly accepted: pulumi.Output<boolean>;
    /**
     * Portfolio identifier.
     */
    readonly portfolioId: pulumi.Output<string>;
    /**
     * Identifier of the principal with whom you will share the portfolio. Valid values AWS account IDs and ARNs of AWS Organizations and organizational units.
     */
    readonly principalId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Enables or disables Principal sharing when creating the portfolio share. If this flag is not provided, principal sharing is disabled.
     */
    readonly sharePrincipals: pulumi.Output<boolean | undefined>;
    /**
     * Whether to enable sharing of `aws.servicecatalog.TagOption` resources when creating the portfolio share.
     */
    readonly shareTagOptions: pulumi.Output<boolean | undefined>;
    /**
     * Type of portfolio share. Valid values are `ACCOUNT` (an external account), `ORGANIZATION` (a share to every account in an organization), `ORGANIZATIONAL_UNIT`, `ORGANIZATION_MEMBER_ACCOUNT` (a share to an account in an organization).
     *
     * The following arguments are optional:
     */
    readonly type: pulumi.Output<string>;
    /**
     * Whether to wait (up to the timeout) for the share to be accepted. Organizational shares are automatically accepted.
     */
    readonly waitForAcceptance: pulumi.Output<boolean | undefined>;
    /**
     * Create a PortfolioShare resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PortfolioShareArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PortfolioShare resources.
 */
export interface PortfolioShareState {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * Whether the shared portfolio is imported by the recipient account. If the recipient is organizational, the share is automatically imported, and the field is always set to true.
     */
    accepted?: pulumi.Input<boolean>;
    /**
     * Portfolio identifier.
     */
    portfolioId?: pulumi.Input<string>;
    /**
     * Identifier of the principal with whom you will share the portfolio. Valid values AWS account IDs and ARNs of AWS Organizations and organizational units.
     */
    principalId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Enables or disables Principal sharing when creating the portfolio share. If this flag is not provided, principal sharing is disabled.
     */
    sharePrincipals?: pulumi.Input<boolean>;
    /**
     * Whether to enable sharing of `aws.servicecatalog.TagOption` resources when creating the portfolio share.
     */
    shareTagOptions?: pulumi.Input<boolean>;
    /**
     * Type of portfolio share. Valid values are `ACCOUNT` (an external account), `ORGANIZATION` (a share to every account in an organization), `ORGANIZATIONAL_UNIT`, `ORGANIZATION_MEMBER_ACCOUNT` (a share to an account in an organization).
     *
     * The following arguments are optional:
     */
    type?: pulumi.Input<string>;
    /**
     * Whether to wait (up to the timeout) for the share to be accepted. Organizational shares are automatically accepted.
     */
    waitForAcceptance?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a PortfolioShare resource.
 */
export interface PortfolioShareArgs {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * Portfolio identifier.
     */
    portfolioId: pulumi.Input<string>;
    /**
     * Identifier of the principal with whom you will share the portfolio. Valid values AWS account IDs and ARNs of AWS Organizations and organizational units.
     */
    principalId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Enables or disables Principal sharing when creating the portfolio share. If this flag is not provided, principal sharing is disabled.
     */
    sharePrincipals?: pulumi.Input<boolean>;
    /**
     * Whether to enable sharing of `aws.servicecatalog.TagOption` resources when creating the portfolio share.
     */
    shareTagOptions?: pulumi.Input<boolean>;
    /**
     * Type of portfolio share. Valid values are `ACCOUNT` (an external account), `ORGANIZATION` (a share to every account in an organization), `ORGANIZATIONAL_UNIT`, `ORGANIZATION_MEMBER_ACCOUNT` (a share to an account in an organization).
     *
     * The following arguments are optional:
     */
    type: pulumi.Input<string>;
    /**
     * Whether to wait (up to the timeout) for the share to be accepted. Organizational shares are automatically accepted.
     */
    waitForAcceptance?: pulumi.Input<boolean>;
}
