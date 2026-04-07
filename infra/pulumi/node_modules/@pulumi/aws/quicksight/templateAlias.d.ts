import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS QuickSight Template Alias.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.quicksight.TemplateAlias("example", {
 *     aliasName: "example-alias",
 *     templateId: test.templateId,
 *     templateVersionNumber: test.versionNumber,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import QuickSight Template Alias using the AWS account ID, template ID, and alias name separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:quicksight/templateAlias:TemplateAlias example 123456789012,example-id,example-alias
 * ```
 */
export declare class TemplateAlias extends pulumi.CustomResource {
    /**
     * Get an existing TemplateAlias resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TemplateAliasState, opts?: pulumi.CustomResourceOptions): TemplateAlias;
    /**
     * Returns true if the given object is an instance of TemplateAlias.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TemplateAlias;
    /**
     * Display name of the template alias.
     */
    readonly aliasName: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the template alias.
     */
    readonly arn: pulumi.Output<string>;
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ID of the template.
     */
    readonly templateId: pulumi.Output<string>;
    /**
     * Version number of the template.
     *
     * The following arguments are optional:
     */
    readonly templateVersionNumber: pulumi.Output<number>;
    /**
     * Create a TemplateAlias resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TemplateAliasArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TemplateAlias resources.
 */
export interface TemplateAliasState {
    /**
     * Display name of the template alias.
     */
    aliasName?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the template alias.
     */
    arn?: pulumi.Input<string>;
    awsAccountId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the template.
     */
    templateId?: pulumi.Input<string>;
    /**
     * Version number of the template.
     *
     * The following arguments are optional:
     */
    templateVersionNumber?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a TemplateAlias resource.
 */
export interface TemplateAliasArgs {
    /**
     * Display name of the template alias.
     */
    aliasName: pulumi.Input<string>;
    awsAccountId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the template.
     */
    templateId: pulumi.Input<string>;
    /**
     * Version number of the template.
     *
     * The following arguments are optional:
     */
    templateVersionNumber: pulumi.Input<number>;
}
