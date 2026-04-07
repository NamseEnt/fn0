import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Service Catalog Tag Option Resource Association.
 *
 * > **Tip:** A "resource" is either a Service Catalog portfolio or product.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.servicecatalog.TagOptionResourceAssociation("example", {
 *     resourceId: "prod-dnigbtea24ste",
 *     tagOptionId: "tag-pjtvyakdlyo3m",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_servicecatalog_tag_option_resource_association` using the tag option ID and resource ID. For example:
 *
 * ```sh
 * $ pulumi import aws:servicecatalog/tagOptionResourceAssociation:TagOptionResourceAssociation example tag-pjtvyakdlyo3m:prod-dnigbtea24ste
 * ```
 */
export declare class TagOptionResourceAssociation extends pulumi.CustomResource {
    /**
     * Get an existing TagOptionResourceAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TagOptionResourceAssociationState, opts?: pulumi.CustomResourceOptions): TagOptionResourceAssociation;
    /**
     * Returns true if the given object is an instance of TagOptionResourceAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TagOptionResourceAssociation;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of the resource.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * Creation time of the resource.
     */
    readonly resourceCreatedTime: pulumi.Output<string>;
    /**
     * Description of the resource.
     */
    readonly resourceDescription: pulumi.Output<string>;
    /**
     * Resource identifier.
     */
    readonly resourceId: pulumi.Output<string>;
    /**
     * Description of the resource.
     */
    readonly resourceName: pulumi.Output<string>;
    /**
     * Tag Option identifier.
     */
    readonly tagOptionId: pulumi.Output<string>;
    /**
     * Create a TagOptionResourceAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TagOptionResourceAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TagOptionResourceAssociation resources.
 */
export interface TagOptionResourceAssociationState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of the resource.
     */
    resourceArn?: pulumi.Input<string>;
    /**
     * Creation time of the resource.
     */
    resourceCreatedTime?: pulumi.Input<string>;
    /**
     * Description of the resource.
     */
    resourceDescription?: pulumi.Input<string>;
    /**
     * Resource identifier.
     */
    resourceId?: pulumi.Input<string>;
    /**
     * Description of the resource.
     */
    resourceName?: pulumi.Input<string>;
    /**
     * Tag Option identifier.
     */
    tagOptionId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TagOptionResourceAssociation resource.
 */
export interface TagOptionResourceAssociationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Resource identifier.
     */
    resourceId: pulumi.Input<string>;
    /**
     * Tag Option identifier.
     */
    tagOptionId: pulumi.Input<string>;
}
