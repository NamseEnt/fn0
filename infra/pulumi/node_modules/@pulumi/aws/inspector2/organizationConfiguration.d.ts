import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an Amazon Inspector Organization Configuration.
 *
 * > **NOTE:** In order for this resource to work, the account you use must be an Inspector Delegated Admin Account.
 *
 * > **NOTE:** When this resource is deleted, EC2, ECR, Lambda, and Lambda code scans will no longer be automatically enabled for new members of your Amazon Inspector organization.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.inspector2.OrganizationConfiguration("example", {autoEnable: {
 *     ec2: true,
 *     ecr: false,
 *     codeRepository: false,
 *     lambda: true,
 *     lambdaCode: true,
 * }});
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
     * Configuration block for auto enabling. See below.
     */
    readonly autoEnable: pulumi.Output<outputs.inspector2.OrganizationConfigurationAutoEnable>;
    /**
     * Whether your configuration reached the max account limit.
     */
    readonly maxAccountLimitReached: pulumi.Output<boolean>;
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
     * Configuration block for auto enabling. See below.
     */
    autoEnable?: pulumi.Input<inputs.inspector2.OrganizationConfigurationAutoEnable>;
    /**
     * Whether your configuration reached the max account limit.
     */
    maxAccountLimitReached?: pulumi.Input<boolean>;
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
     * Configuration block for auto enabling. See below.
     */
    autoEnable: pulumi.Input<inputs.inspector2.OrganizationConfigurationAutoEnable>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
