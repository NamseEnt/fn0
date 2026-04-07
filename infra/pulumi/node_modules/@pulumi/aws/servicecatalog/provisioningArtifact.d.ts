import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Service Catalog Provisioning Artifact for a specified product.
 *
 * > A "provisioning artifact" is also referred to as a "version."
 *
 * > **NOTE:** You cannot create a provisioning artifact for a product that was shared with you.
 *
 * > **NOTE:** The user or role that use this resource must have the `cloudformation:GetTemplate` IAM policy permission. This policy permission is required when using the `templatePhysicalId` argument.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicecatalog.ProvisioningArtifact("example", {
 *     name: "example",
 *     productId: exampleAwsServicecatalogProduct.id,
 *     type: "CLOUD_FORMATION_TEMPLATE",
 *     templateUrl: `https://${exampleAwsS3Bucket.bucketRegionalDomainName}/${exampleAwsS3Object.key}`,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_servicecatalog_provisioning_artifact` using the provisioning artifact ID and product ID separated by a colon. For example:
 *
 * ```sh
 * $ pulumi import aws:servicecatalog/provisioningArtifact:ProvisioningArtifact example pa-ij2b6lusy6dec:prod-el3an0rma3
 * ```
 */
export declare class ProvisioningArtifact extends pulumi.CustomResource {
    /**
     * Get an existing ProvisioningArtifact resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProvisioningArtifactState, opts?: pulumi.CustomResourceOptions): ProvisioningArtifact;
    /**
     * Returns true if the given object is an instance of ProvisioningArtifact.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ProvisioningArtifact;
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). The default value is `en`.
     */
    readonly acceptLanguage: pulumi.Output<string | undefined>;
    /**
     * Whether the product version is active. Inactive provisioning artifacts are invisible to end users. End users cannot launch or update a provisioned product from an inactive provisioning artifact. Default is `true`.
     */
    readonly active: pulumi.Output<boolean | undefined>;
    /**
     * Time when the provisioning artifact was created.
     */
    readonly createdTime: pulumi.Output<string>;
    /**
     * Description of the provisioning artifact (i.e., version), including how it differs from the previous provisioning artifact.
     */
    readonly description: pulumi.Output<string>;
    /**
     * Whether AWS Service Catalog stops validating the specified provisioning artifact template even if it is invalid.
     */
    readonly disableTemplateValidation: pulumi.Output<boolean | undefined>;
    /**
     * Information set by the administrator to provide guidance to end users about which provisioning artifacts to use. Valid values are `DEFAULT` and `DEPRECATED`. The default is `DEFAULT`. Users are able to make updates to a provisioned product of a deprecated version but cannot launch new provisioned products using a deprecated version.
     */
    readonly guidance: pulumi.Output<string | undefined>;
    /**
     * Name of the provisioning artifact (for example, `v1`, `v2beta`). No spaces are allowed.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Identifier of the product.
     */
    readonly productId: pulumi.Output<string>;
    /**
     * Provisioning artifact identifier.
     */
    readonly provisioningArtifactId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Template source as the physical ID of the resource that contains the template. Currently only supports CloudFormation stack ARN. Specify the physical ID as `arn:[partition]:cloudformation:[region]:[account ID]:stack/[stack name]/[resource ID]`.
     */
    readonly templatePhysicalId: pulumi.Output<string | undefined>;
    /**
     * Template source as URL of the CloudFormation template in Amazon S3.
     *
     * The following arguments are optional:
     */
    readonly templateUrl: pulumi.Output<string | undefined>;
    /**
     * Type of provisioning artifact. See [AWS Docs](https://docs.aws.amazon.com/servicecatalog/latest/dg/API_ProvisioningArtifactProperties.html) for valid list of values.
     */
    readonly type: pulumi.Output<string | undefined>;
    /**
     * Create a ProvisioningArtifact resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ProvisioningArtifactArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ProvisioningArtifact resources.
 */
export interface ProvisioningArtifactState {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). The default value is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * Whether the product version is active. Inactive provisioning artifacts are invisible to end users. End users cannot launch or update a provisioned product from an inactive provisioning artifact. Default is `true`.
     */
    active?: pulumi.Input<boolean>;
    /**
     * Time when the provisioning artifact was created.
     */
    createdTime?: pulumi.Input<string>;
    /**
     * Description of the provisioning artifact (i.e., version), including how it differs from the previous provisioning artifact.
     */
    description?: pulumi.Input<string>;
    /**
     * Whether AWS Service Catalog stops validating the specified provisioning artifact template even if it is invalid.
     */
    disableTemplateValidation?: pulumi.Input<boolean>;
    /**
     * Information set by the administrator to provide guidance to end users about which provisioning artifacts to use. Valid values are `DEFAULT` and `DEPRECATED`. The default is `DEFAULT`. Users are able to make updates to a provisioned product of a deprecated version but cannot launch new provisioned products using a deprecated version.
     */
    guidance?: pulumi.Input<string>;
    /**
     * Name of the provisioning artifact (for example, `v1`, `v2beta`). No spaces are allowed.
     */
    name?: pulumi.Input<string>;
    /**
     * Identifier of the product.
     */
    productId?: pulumi.Input<string>;
    /**
     * Provisioning artifact identifier.
     */
    provisioningArtifactId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Template source as the physical ID of the resource that contains the template. Currently only supports CloudFormation stack ARN. Specify the physical ID as `arn:[partition]:cloudformation:[region]:[account ID]:stack/[stack name]/[resource ID]`.
     */
    templatePhysicalId?: pulumi.Input<string>;
    /**
     * Template source as URL of the CloudFormation template in Amazon S3.
     *
     * The following arguments are optional:
     */
    templateUrl?: pulumi.Input<string>;
    /**
     * Type of provisioning artifact. See [AWS Docs](https://docs.aws.amazon.com/servicecatalog/latest/dg/API_ProvisioningArtifactProperties.html) for valid list of values.
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ProvisioningArtifact resource.
 */
export interface ProvisioningArtifactArgs {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). The default value is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * Whether the product version is active. Inactive provisioning artifacts are invisible to end users. End users cannot launch or update a provisioned product from an inactive provisioning artifact. Default is `true`.
     */
    active?: pulumi.Input<boolean>;
    /**
     * Description of the provisioning artifact (i.e., version), including how it differs from the previous provisioning artifact.
     */
    description?: pulumi.Input<string>;
    /**
     * Whether AWS Service Catalog stops validating the specified provisioning artifact template even if it is invalid.
     */
    disableTemplateValidation?: pulumi.Input<boolean>;
    /**
     * Information set by the administrator to provide guidance to end users about which provisioning artifacts to use. Valid values are `DEFAULT` and `DEPRECATED`. The default is `DEFAULT`. Users are able to make updates to a provisioned product of a deprecated version but cannot launch new provisioned products using a deprecated version.
     */
    guidance?: pulumi.Input<string>;
    /**
     * Name of the provisioning artifact (for example, `v1`, `v2beta`). No spaces are allowed.
     */
    name?: pulumi.Input<string>;
    /**
     * Identifier of the product.
     */
    productId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Template source as the physical ID of the resource that contains the template. Currently only supports CloudFormation stack ARN. Specify the physical ID as `arn:[partition]:cloudformation:[region]:[account ID]:stack/[stack name]/[resource ID]`.
     */
    templatePhysicalId?: pulumi.Input<string>;
    /**
     * Template source as URL of the CloudFormation template in Amazon S3.
     *
     * The following arguments are optional:
     */
    templateUrl?: pulumi.Input<string>;
    /**
     * Type of provisioning artifact. See [AWS Docs](https://docs.aws.amazon.com/servicecatalog/latest/dg/API_ProvisioningArtifactProperties.html) for valid list of values.
     */
    type?: pulumi.Input<string>;
}
