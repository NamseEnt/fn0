import * as pulumi from "@pulumi/pulumi";
/**
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.detective.Graph("example", {enable: true});
 * const exampleOrganizationConfiguration = new aws.detective.OrganizationConfiguration("example", {
 *     autoEnable: true,
 *     graphArn: example.graphArn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_detective_organization_admin_account` using the behavior graph ARN. For example:
 *
 * ```sh
 * $ pulumi import aws:detective/organizationConfiguration:OrganizationConfiguration example arn:aws:detective:us-east-1:123456789012:graph:00b00fd5aecc0ab60a708659477e9617
 * ```
 */
export declare class OrganizationConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing OrganizationConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: OrganizationConfigurationState, opts?: pulumi.CustomResourceOptions): OrganizationConfiguration;
    /**
     * Returns true if the given object is an instance of OrganizationConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is OrganizationConfiguration;
    /**
     * When this setting is enabled, all new accounts that are created in, or added to, the organization are added as a member accounts of the organization’s Detective delegated administrator and Detective is enabled in that AWS Region.
     */
    readonly autoEnable: pulumi.Output<boolean>;
    /**
     * ARN of the behavior graph.
     */
    readonly graphArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a OrganizationConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: OrganizationConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering OrganizationConfiguration resources.
 */
export interface OrganizationConfigurationState {
    /**
     * When this setting is enabled, all new accounts that are created in, or added to, the organization are added as a member accounts of the organization’s Detective delegated administrator and Detective is enabled in that AWS Region.
     */
    autoEnable?: pulumi.Input<boolean>;
    /**
     * ARN of the behavior graph.
     */
    graphArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a OrganizationConfiguration resource.
 */
export interface OrganizationConfigurationArgs {
    /**
     * When this setting is enabled, all new accounts that are created in, or added to, the organization are added as a member accounts of the organization’s Detective delegated administrator and Detective is enabled in that AWS Region.
     */
    autoEnable: pulumi.Input<boolean>;
    /**
     * ARN of the behavior graph.
     */
    graphArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
