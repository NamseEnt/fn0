import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Service Quotas Template Association.
 *
 * > Only the management account of an organization can associate Service Quota templates, and this must be done from the `us-east-1` region.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicequotas.TemplateAssociation("example", {});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Service Quotas Template Association using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:servicequotas/templateAssociation:TemplateAssociation example 123456789012
 * ```
 */
export declare class TemplateAssociation extends pulumi.CustomResource {
    /**
     * Get an existing TemplateAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TemplateAssociationState, opts?: pulumi.CustomResourceOptions): TemplateAssociation;
    /**
     * Returns true if the given object is an instance of TemplateAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TemplateAssociation;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly skipDestroy: pulumi.Output<boolean | undefined>;
    /**
     * Association status. Creating this resource will result in an `ASSOCIATED` status, and quota increase requests in the template are automatically applied to new AWS accounts in the organization.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Create a TemplateAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: TemplateAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TemplateAssociation resources.
 */
export interface TemplateAssociationState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    skipDestroy?: pulumi.Input<boolean>;
    /**
     * Association status. Creating this resource will result in an `ASSOCIATED` status, and quota increase requests in the template are automatically applied to new AWS accounts in the organization.
     */
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TemplateAssociation resource.
 */
export interface TemplateAssociationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    skipDestroy?: pulumi.Input<boolean>;
}
