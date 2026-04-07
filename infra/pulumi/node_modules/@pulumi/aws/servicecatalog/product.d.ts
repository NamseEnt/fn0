import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Service Catalog Product.
 *
 * > **NOTE:** The user or role that uses this resources must have the `cloudformation:GetTemplate` IAM policy permission. This policy permission is required when using the `templatePhysicalId` argument.
 *
 * > A "provisioning artifact" is also referred to as a "version." A "distributor" is also referred to as a "vendor."
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicecatalog.Product("example", {
 *     name: "example",
 *     owner: "example-owner",
 *     type: "CLOUD_FORMATION_TEMPLATE",
 *     provisioningArtifactParameters: {
 *         templateUrl: "https://s3.amazonaws.com/cf-templates-ozkq9d3hgiq2-us-east-1/temp1.json",
 *     },
 *     tags: {
 *         foo: "bar",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_servicecatalog_product` using the product ID. For example:
 *
 * ```sh
 * $ pulumi import aws:servicecatalog/product:Product example prod-dnigbtea24ste
 * ```
 */
export declare class Product extends pulumi.CustomResource {
    /**
     * Get an existing Product resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProductState, opts?: pulumi.CustomResourceOptions): Product;
    /**
     * Returns true if the given object is an instance of Product.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Product;
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    readonly acceptLanguage: pulumi.Output<string | undefined>;
    /**
     * ARN of the product.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Time when the product was created.
     */
    readonly createdTime: pulumi.Output<string>;
    /**
     * Description of the product.
     */
    readonly description: pulumi.Output<string>;
    /**
     * Distributor (i.e., vendor) of the product.
     */
    readonly distributor: pulumi.Output<string>;
    /**
     * Whether the product has a default path. If the product does not have a default path, call `ListLaunchPaths` to disambiguate between paths.  Otherwise, `ListLaunchPaths` is not required, and the output of ProductViewSummary can be used directly with `DescribeProvisioningParameters`.
     */
    readonly hasDefaultPath: pulumi.Output<boolean>;
    /**
     * Name of the product.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Owner of the product.
     */
    readonly owner: pulumi.Output<string>;
    /**
     * Configuration block for provisioning artifact (i.e., version) parameters. See `provisioningArtifactParameters` Block for details.
     */
    readonly provisioningArtifactParameters: pulumi.Output<outputs.servicecatalog.ProductProvisioningArtifactParameters>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Status of the product.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Support information about the product.
     */
    readonly supportDescription: pulumi.Output<string>;
    /**
     * Contact email for product support.
     */
    readonly supportEmail: pulumi.Output<string>;
    /**
     * Contact URL for product support.
     */
    readonly supportUrl: pulumi.Output<string>;
    /**
     * Tags to apply to the product. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Type of product. See [AWS Docs](https://docs.aws.amazon.com/servicecatalog/latest/dg/API_CreateProduct.html#API_CreateProduct_RequestSyntax) for valid list of values.
     *
     * The following arguments are optional:
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a Product resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ProductArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Product resources.
 */
export interface ProductState {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * ARN of the product.
     */
    arn?: pulumi.Input<string>;
    /**
     * Time when the product was created.
     */
    createdTime?: pulumi.Input<string>;
    /**
     * Description of the product.
     */
    description?: pulumi.Input<string>;
    /**
     * Distributor (i.e., vendor) of the product.
     */
    distributor?: pulumi.Input<string>;
    /**
     * Whether the product has a default path. If the product does not have a default path, call `ListLaunchPaths` to disambiguate between paths.  Otherwise, `ListLaunchPaths` is not required, and the output of ProductViewSummary can be used directly with `DescribeProvisioningParameters`.
     */
    hasDefaultPath?: pulumi.Input<boolean>;
    /**
     * Name of the product.
     */
    name?: pulumi.Input<string>;
    /**
     * Owner of the product.
     */
    owner?: pulumi.Input<string>;
    /**
     * Configuration block for provisioning artifact (i.e., version) parameters. See `provisioningArtifactParameters` Block for details.
     */
    provisioningArtifactParameters?: pulumi.Input<inputs.servicecatalog.ProductProvisioningArtifactParameters>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Status of the product.
     */
    status?: pulumi.Input<string>;
    /**
     * Support information about the product.
     */
    supportDescription?: pulumi.Input<string>;
    /**
     * Contact email for product support.
     */
    supportEmail?: pulumi.Input<string>;
    /**
     * Contact URL for product support.
     */
    supportUrl?: pulumi.Input<string>;
    /**
     * Tags to apply to the product. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Type of product. See [AWS Docs](https://docs.aws.amazon.com/servicecatalog/latest/dg/API_CreateProduct.html#API_CreateProduct_RequestSyntax) for valid list of values.
     *
     * The following arguments are optional:
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Product resource.
 */
export interface ProductArgs {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * Description of the product.
     */
    description?: pulumi.Input<string>;
    /**
     * Distributor (i.e., vendor) of the product.
     */
    distributor?: pulumi.Input<string>;
    /**
     * Name of the product.
     */
    name?: pulumi.Input<string>;
    /**
     * Owner of the product.
     */
    owner: pulumi.Input<string>;
    /**
     * Configuration block for provisioning artifact (i.e., version) parameters. See `provisioningArtifactParameters` Block for details.
     */
    provisioningArtifactParameters: pulumi.Input<inputs.servicecatalog.ProductProvisioningArtifactParameters>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Support information about the product.
     */
    supportDescription?: pulumi.Input<string>;
    /**
     * Contact email for product support.
     */
    supportEmail?: pulumi.Input<string>;
    /**
     * Contact URL for product support.
     */
    supportUrl?: pulumi.Input<string>;
    /**
     * Tags to apply to the product. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Type of product. See [AWS Docs](https://docs.aws.amazon.com/servicecatalog/latest/dg/API_CreateProduct.html#API_CreateProduct_RequestSyntax) for valid list of values.
     *
     * The following arguments are optional:
     */
    type: pulumi.Input<string>;
}
