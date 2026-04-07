import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Service Catalog Product Portfolio Association.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicecatalog.ProductPortfolioAssociation("example", {
 *     portfolioId: "port-68656c6c6f",
 *     productId: "prod-dnigbtea24ste",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_servicecatalog_product_portfolio_association` using the accept language, portfolio ID, and product ID. For example:
 *
 * ```sh
 * $ pulumi import aws:servicecatalog/productPortfolioAssociation:ProductPortfolioAssociation example en:port-68656c6c6f:prod-dnigbtea24ste
 * ```
 */
export declare class ProductPortfolioAssociation extends pulumi.CustomResource {
    /**
     * Get an existing ProductPortfolioAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProductPortfolioAssociationState, opts?: pulumi.CustomResourceOptions): ProductPortfolioAssociation;
    /**
     * Returns true if the given object is an instance of ProductPortfolioAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ProductPortfolioAssociation;
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    readonly acceptLanguage: pulumi.Output<string | undefined>;
    /**
     * Portfolio identifier.
     */
    readonly portfolioId: pulumi.Output<string>;
    /**
     * Product identifier.
     *
     * The following arguments are optional:
     */
    readonly productId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Identifier of the source portfolio.
     */
    readonly sourcePortfolioId: pulumi.Output<string | undefined>;
    /**
     * Create a ProductPortfolioAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ProductPortfolioAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ProductPortfolioAssociation resources.
 */
export interface ProductPortfolioAssociationState {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * Portfolio identifier.
     */
    portfolioId?: pulumi.Input<string>;
    /**
     * Product identifier.
     *
     * The following arguments are optional:
     */
    productId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of the source portfolio.
     */
    sourcePortfolioId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ProductPortfolioAssociation resource.
 */
export interface ProductPortfolioAssociationArgs {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * Portfolio identifier.
     */
    portfolioId: pulumi.Input<string>;
    /**
     * Product identifier.
     *
     * The following arguments are optional:
     */
    productId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Identifier of the source portfolio.
     */
    sourcePortfolioId?: pulumi.Input<string>;
}
