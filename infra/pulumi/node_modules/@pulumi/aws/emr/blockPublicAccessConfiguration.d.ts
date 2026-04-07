import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS EMR block public access configuration. This region level security configuration restricts the launch of EMR clusters that have associated security groups permitting public access on unspecified ports. See the [EMR Block Public Access Configuration](https://docs.aws.amazon.com/emr/latest/ManagementGuide/emr-block-public-access.html) documentation for further information.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.emr.BlockPublicAccessConfiguration("example", {blockPublicSecurityGroupRules: true});
 * ```
 *
 * ### Default Configuration
 *
 * By default, each AWS region is equipped with a block public access configuration that prevents EMR clusters from being launched if they have security group rules permitting public access on any port except for port 22. The default configuration can be managed using this resource.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.emr.BlockPublicAccessConfiguration("example", {
 *     blockPublicSecurityGroupRules: true,
 *     permittedPublicSecurityGroupRuleRanges: [{
 *         minRange: 22,
 *         maxRange: 22,
 *     }],
 * });
 * ```
 *
 * > **NOTE:** If an `aws.emr.BlockPublicAccessConfiguration` resource is destroyed, the configuration will reset to this default configuration.
 *
 * ### Multiple Permitted Public Security Group Rule Ranges
 *
 * The resource permits specification of multiple `permittedPublicSecurityGroupRuleRange` blocks.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.emr.BlockPublicAccessConfiguration("example", {
 *     blockPublicSecurityGroupRules: true,
 *     permittedPublicSecurityGroupRuleRanges: [
 *         {
 *             minRange: 22,
 *             maxRange: 22,
 *         },
 *         {
 *             minRange: 100,
 *             maxRange: 101,
 *         },
 *     ],
 * });
 * ```
 *
 * ### Disabling Block Public Access
 *
 * To permit EMR clusters to be launched in the configured region regardless of associated security group rules, the Block Public Access feature can be disabled using this resource.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.emr.BlockPublicAccessConfiguration("example", {blockPublicSecurityGroupRules: false});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import the current EMR Block Public Access Configuration. For example:
 *
 * ```sh
 * $ pulumi import aws:emr/blockPublicAccessConfiguration:BlockPublicAccessConfiguration example current
 * ```
 */
export declare class BlockPublicAccessConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing BlockPublicAccessConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BlockPublicAccessConfigurationState, opts?: pulumi.CustomResourceOptions): BlockPublicAccessConfiguration;
    /**
     * Returns true if the given object is an instance of BlockPublicAccessConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BlockPublicAccessConfiguration;
    /**
     * Enable or disable EMR Block Public Access.
     *
     * The following arguments are optional:
     */
    readonly blockPublicSecurityGroupRules: pulumi.Output<boolean>;
    /**
     * Configuration block for defining permitted public security group rule port ranges. Can be defined multiple times per resource. Only valid if `blockPublicSecurityGroupRules` is set to `true`.
     */
    readonly permittedPublicSecurityGroupRuleRanges: pulumi.Output<outputs.emr.BlockPublicAccessConfigurationPermittedPublicSecurityGroupRuleRange[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a BlockPublicAccessConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: BlockPublicAccessConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BlockPublicAccessConfiguration resources.
 */
export interface BlockPublicAccessConfigurationState {
    /**
     * Enable or disable EMR Block Public Access.
     *
     * The following arguments are optional:
     */
    blockPublicSecurityGroupRules?: pulumi.Input<boolean>;
    /**
     * Configuration block for defining permitted public security group rule port ranges. Can be defined multiple times per resource. Only valid if `blockPublicSecurityGroupRules` is set to `true`.
     */
    permittedPublicSecurityGroupRuleRanges?: pulumi.Input<pulumi.Input<inputs.emr.BlockPublicAccessConfigurationPermittedPublicSecurityGroupRuleRange>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a BlockPublicAccessConfiguration resource.
 */
export interface BlockPublicAccessConfigurationArgs {
    /**
     * Enable or disable EMR Block Public Access.
     *
     * The following arguments are optional:
     */
    blockPublicSecurityGroupRules: pulumi.Input<boolean>;
    /**
     * Configuration block for defining permitted public security group rule port ranges. Can be defined multiple times per resource. Only valid if `blockPublicSecurityGroupRules` is set to `true`.
     */
    permittedPublicSecurityGroupRuleRanges?: pulumi.Input<pulumi.Input<inputs.emr.BlockPublicAccessConfigurationPermittedPublicSecurityGroupRuleRange>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
