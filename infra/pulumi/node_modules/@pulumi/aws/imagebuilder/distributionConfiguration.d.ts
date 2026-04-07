import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an Image Builder Distribution Configuration.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.imagebuilder.DistributionConfiguration("example", {
 *     name: "example",
 *     distributions: [{
 *         amiDistributionConfiguration: {
 *             amiTags: {
 *                 CostCenter: "IT",
 *             },
 *             name: "example-{{ imagebuilder:buildDate }}",
 *             launchPermission: {
 *                 userIds: ["123456789012"],
 *             },
 *         },
 *         launchTemplateConfigurations: [{
 *             launchTemplateId: "lt-0aaa1bcde2ff3456",
 *         }],
 *         region: "us-east-1",
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Image Builder distribution configuration.
 *
 * Using `pulumi import`, import `aws_imagebuilder_distribution_configurations` resources using the Amazon Resource Name (ARN). For example:
 *
 * console
 *
 * % pulumi import aws_imagebuilder_distribution_configuration.example arn:aws:imagebuilder:us-east-1:123456789012:distribution-configuration/example
 */
export declare class DistributionConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing DistributionConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DistributionConfigurationState, opts?: pulumi.CustomResourceOptions): DistributionConfiguration;
    /**
     * Returns true if the given object is an instance of DistributionConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DistributionConfiguration;
    /**
     * (Required) Amazon Resource Name (ARN) of the distribution configuration.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Date the distribution configuration was created.
     */
    readonly dateCreated: pulumi.Output<string>;
    /**
     * Date the distribution configuration was updated.
     */
    readonly dateUpdated: pulumi.Output<string>;
    /**
     * Description of the distribution configuration.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * One or more configuration blocks with distribution settings. Detailed below.
     *
     * The following arguments are optional:
     */
    readonly distributions: pulumi.Output<outputs.imagebuilder.DistributionConfigurationDistribution[]>;
    /**
     * Name of the distribution configuration.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of resource tags for the distribution configuration. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a DistributionConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DistributionConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DistributionConfiguration resources.
 */
export interface DistributionConfigurationState {
    /**
     * (Required) Amazon Resource Name (ARN) of the distribution configuration.
     */
    arn?: pulumi.Input<string>;
    /**
     * Date the distribution configuration was created.
     */
    dateCreated?: pulumi.Input<string>;
    /**
     * Date the distribution configuration was updated.
     */
    dateUpdated?: pulumi.Input<string>;
    /**
     * Description of the distribution configuration.
     */
    description?: pulumi.Input<string>;
    /**
     * One or more configuration blocks with distribution settings. Detailed below.
     *
     * The following arguments are optional:
     */
    distributions?: pulumi.Input<pulumi.Input<inputs.imagebuilder.DistributionConfigurationDistribution>[]>;
    /**
     * Name of the distribution configuration.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags for the distribution configuration. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
}
/**
 * The set of arguments for constructing a DistributionConfiguration resource.
 */
export interface DistributionConfigurationArgs {
    /**
     * Description of the distribution configuration.
     */
    description?: pulumi.Input<string>;
    /**
     * One or more configuration blocks with distribution settings. Detailed below.
     *
     * The following arguments are optional:
     */
    distributions: pulumi.Input<pulumi.Input<inputs.imagebuilder.DistributionConfigurationDistribution>[]>;
    /**
     * Name of the distribution configuration.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags for the distribution configuration. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
