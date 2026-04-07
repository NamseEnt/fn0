import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Service Catalog AppRegistry Attribute Group Association.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicecatalog.AppregistryApplication("example", {name: "example-app"});
 * const exampleAppregistryAttributeGroup = new aws.servicecatalog.AppregistryAttributeGroup("example", {
 *     name: "example",
 *     description: "example description",
 *     attributes: JSON.stringify({
 *         app: "exampleapp",
 *         group: "examplegroup",
 *     }),
 * });
 * const exampleAppregistryAttributeGroupAssociation = new aws.servicecatalog.AppregistryAttributeGroupAssociation("example", {
 *     applicationId: example.id,
 *     attributeGroupId: exampleAppregistryAttributeGroup.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Service Catalog AppRegistry Attribute Group Association using `application_id` and `attribute_group_id` arguments separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:servicecatalog/appregistryAttributeGroupAssociation:AppregistryAttributeGroupAssociation example 12456778723424sdffsdfsdq34,12234t3564dsfsdf34asff4ww3
 * ```
 */
export declare class AppregistryAttributeGroupAssociation extends pulumi.CustomResource {
    /**
     * Get an existing AppregistryAttributeGroupAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AppregistryAttributeGroupAssociationState, opts?: pulumi.CustomResourceOptions): AppregistryAttributeGroupAssociation;
    /**
     * Returns true if the given object is an instance of AppregistryAttributeGroupAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AppregistryAttributeGroupAssociation;
    /**
     * ID of the application.
     */
    readonly applicationId: pulumi.Output<string>;
    /**
     * ID of the attribute group to associate with the application.
     */
    readonly attributeGroupId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a AppregistryAttributeGroupAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AppregistryAttributeGroupAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AppregistryAttributeGroupAssociation resources.
 */
export interface AppregistryAttributeGroupAssociationState {
    /**
     * ID of the application.
     */
    applicationId?: pulumi.Input<string>;
    /**
     * ID of the attribute group to associate with the application.
     */
    attributeGroupId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AppregistryAttributeGroupAssociation resource.
 */
export interface AppregistryAttributeGroupAssociationArgs {
    /**
     * ID of the application.
     */
    applicationId: pulumi.Input<string>;
    /**
     * ID of the attribute group to associate with the application.
     */
    attributeGroupId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
