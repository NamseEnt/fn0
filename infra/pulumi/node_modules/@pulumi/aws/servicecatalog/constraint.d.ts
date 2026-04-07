import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Service Catalog Constraint.
 *
 * > **NOTE:** This resource does not associate a Service Catalog product and portfolio. However, the product and portfolio must be associated (see the `aws.servicecatalog.ProductPortfolioAssociation` resource) prior to creating a constraint or you will receive an error.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicecatalog.Constraint("example", {
 *     description: "Back off, man. I'm a scientist.",
 *     portfolioId: exampleAwsServicecatalogPortfolio.id,
 *     productId: exampleAwsServicecatalogProduct.id,
 *     type: "LAUNCH",
 *     parameters: JSON.stringify({
 *         RoleArn: "arn:aws:iam::123456789012:role/LaunchRole",
 *     }),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_servicecatalog_constraint` using the constraint ID. For example:
 *
 * ```sh
 * $ pulumi import aws:servicecatalog/constraint:Constraint example cons-nmdkb6cgxfcrs
 * ```
 */
export declare class Constraint extends pulumi.CustomResource {
    /**
     * Get an existing Constraint resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConstraintState, opts?: pulumi.CustomResourceOptions): Constraint;
    /**
     * Returns true if the given object is an instance of Constraint.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Constraint;
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    readonly acceptLanguage: pulumi.Output<string | undefined>;
    /**
     * Description of the constraint.
     */
    readonly description: pulumi.Output<string>;
    /**
     * Owner of the constraint.
     */
    readonly owner: pulumi.Output<string>;
    /**
     * Constraint parameters in JSON format. The syntax depends on the constraint type. See details below.
     */
    readonly parameters: pulumi.Output<string>;
    /**
     * Portfolio identifier.
     */
    readonly portfolioId: pulumi.Output<string>;
    /**
     * Product identifier.
     */
    readonly productId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly status: pulumi.Output<string>;
    /**
     * Type of constraint. Valid values are `LAUNCH`, `NOTIFICATION`, `RESOURCE_UPDATE`, `STACKSET`, and `TEMPLATE`.
     *
     * The following arguments are optional:
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a Constraint resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ConstraintArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Constraint resources.
 */
export interface ConstraintState {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * Description of the constraint.
     */
    description?: pulumi.Input<string>;
    /**
     * Owner of the constraint.
     */
    owner?: pulumi.Input<string>;
    /**
     * Constraint parameters in JSON format. The syntax depends on the constraint type. See details below.
     */
    parameters?: pulumi.Input<string>;
    /**
     * Portfolio identifier.
     */
    portfolioId?: pulumi.Input<string>;
    /**
     * Product identifier.
     */
    productId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    status?: pulumi.Input<string>;
    /**
     * Type of constraint. Valid values are `LAUNCH`, `NOTIFICATION`, `RESOURCE_UPDATE`, `STACKSET`, and `TEMPLATE`.
     *
     * The following arguments are optional:
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Constraint resource.
 */
export interface ConstraintArgs {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * Description of the constraint.
     */
    description?: pulumi.Input<string>;
    /**
     * Constraint parameters in JSON format. The syntax depends on the constraint type. See details below.
     */
    parameters: pulumi.Input<string>;
    /**
     * Portfolio identifier.
     */
    portfolioId: pulumi.Input<string>;
    /**
     * Product identifier.
     */
    productId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Type of constraint. Valid values are `LAUNCH`, `NOTIFICATION`, `RESOURCE_UPDATE`, `STACKSET`, and `TEMPLATE`.
     *
     * The following arguments are optional:
     */
    type: pulumi.Input<string>;
}
