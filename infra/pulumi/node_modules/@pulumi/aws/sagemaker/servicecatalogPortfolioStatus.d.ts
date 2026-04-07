import * as pulumi from "@pulumi/pulumi";
/**
 * Manages status of Service Catalog in SageMaker. Service Catalog is used to create SageMaker AI projects.
 *
 * ## Example Usage
 *
 * Usage:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sagemaker.ServicecatalogPortfolioStatus("example", {status: "Enabled"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import models using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/servicecatalogPortfolioStatus:ServicecatalogPortfolioStatus example us-east-1
 * ```
 */
export declare class ServicecatalogPortfolioStatus extends pulumi.CustomResource {
    /**
     * Get an existing ServicecatalogPortfolioStatus resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ServicecatalogPortfolioStatusState, opts?: pulumi.CustomResourceOptions): ServicecatalogPortfolioStatus;
    /**
     * Returns true if the given object is an instance of ServicecatalogPortfolioStatus.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ServicecatalogPortfolioStatus;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Whether Service Catalog is enabled or disabled in SageMaker. Valid values are `Enabled` and `Disabled`.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Create a ServicecatalogPortfolioStatus resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ServicecatalogPortfolioStatusArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ServicecatalogPortfolioStatus resources.
 */
export interface ServicecatalogPortfolioStatusState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether Service Catalog is enabled or disabled in SageMaker. Valid values are `Enabled` and `Disabled`.
     */
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ServicecatalogPortfolioStatus resource.
 */
export interface ServicecatalogPortfolioStatusArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Whether Service Catalog is enabled or disabled in SageMaker. Valid values are `Enabled` and `Disabled`.
     */
    status: pulumi.Input<string>;
}
