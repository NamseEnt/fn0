import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Service Catalog Principal Portfolio Association.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicecatalog.PrincipalPortfolioAssociation("example", {
 *     portfolioId: "port-68656c6c6f",
 *     principalArn: "arn:aws:iam::123456789012:user/Eleanor",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_servicecatalog_principal_portfolio_association` using `accept_language`, `principal_arn`, `portfolio_id`, and `principal_type` separated by a comma. For example:
 *
 * ```sh
 * $ pulumi import aws:servicecatalog/principalPortfolioAssociation:PrincipalPortfolioAssociation example en,arn:aws:iam::123456789012:user/Eleanor,port-68656c6c6f,IAM
 * ```
 */
export declare class PrincipalPortfolioAssociation extends pulumi.CustomResource {
    /**
     * Get an existing PrincipalPortfolioAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PrincipalPortfolioAssociationState, opts?: pulumi.CustomResourceOptions): PrincipalPortfolioAssociation;
    /**
     * Returns true if the given object is an instance of PrincipalPortfolioAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PrincipalPortfolioAssociation;
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    readonly acceptLanguage: pulumi.Output<string | undefined>;
    /**
     * Portfolio identifier.
     */
    readonly portfolioId: pulumi.Output<string>;
    /**
     * Principal ARN.
     *
     * The following arguments are optional:
     */
    readonly principalArn: pulumi.Output<string>;
    /**
     * Principal type. Setting this argument empty (e.g., `principalType = ""`) will result in an error. Valid values are `IAM` and `IAM_PATTERN`. Default is `IAM`.
     */
    readonly principalType: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a PrincipalPortfolioAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PrincipalPortfolioAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PrincipalPortfolioAssociation resources.
 */
export interface PrincipalPortfolioAssociationState {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * Portfolio identifier.
     */
    portfolioId?: pulumi.Input<string>;
    /**
     * Principal ARN.
     *
     * The following arguments are optional:
     */
    principalArn?: pulumi.Input<string>;
    /**
     * Principal type. Setting this argument empty (e.g., `principalType = ""`) will result in an error. Valid values are `IAM` and `IAM_PATTERN`. Default is `IAM`.
     */
    principalType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PrincipalPortfolioAssociation resource.
 */
export interface PrincipalPortfolioAssociationArgs {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * Portfolio identifier.
     */
    portfolioId: pulumi.Input<string>;
    /**
     * Principal ARN.
     *
     * The following arguments are optional:
     */
    principalArn: pulumi.Input<string>;
    /**
     * Principal type. Setting this argument empty (e.g., `principalType = ""`) will result in an error. Valid values are `IAM` and `IAM_PATTERN`. Default is `IAM`.
     */
    principalType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
