import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages Security Hub configuration policy
 *
 * > **NOTE:** This resource requires `aws.securityhub.OrganizationConfiguration` to be configured of type `CENTRAL`. More information about Security Hub central configuration and configuration policies can be found in the [How Security Hub configuration policies work](https://docs.aws.amazon.com/securityhub/latest/userguide/configuration-policies-overview.html) documentation.
 *
 * ## Example Usage
 *
 * ### Default standards enabled
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.securityhub.FindingAggregator("example", {linkingMode: "ALL_REGIONS"});
 * const exampleOrganizationConfiguration = new aws.securityhub.OrganizationConfiguration("example", {
 *     autoEnable: false,
 *     autoEnableStandards: "NONE",
 *     organizationConfiguration: {
 *         configurationType: "CENTRAL",
 *     },
 * }, {
 *     dependsOn: [example],
 * });
 * const exampleConfigurationPolicy = new aws.securityhub.ConfigurationPolicy("example", {
 *     name: "Example",
 *     description: "This is an example configuration policy",
 *     configurationPolicy: {
 *         serviceEnabled: true,
 *         enabledStandardArns: [
 *             "arn:aws:securityhub:us-east-1::standards/aws-foundational-security-best-practices/v/1.0.0",
 *             "arn:aws:securityhub:::ruleset/cis-aws-foundations-benchmark/v/1.2.0",
 *         ],
 *         securityControlsConfiguration: {
 *             disabledControlIdentifiers: [],
 *         },
 *     },
 * }, {
 *     dependsOn: [exampleOrganizationConfiguration],
 * });
 * ```
 *
 * ### Disabled Policy
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const disabled = new aws.securityhub.ConfigurationPolicy("disabled", {
 *     name: "Disabled",
 *     description: "This is an example of disabled configuration policy",
 *     configurationPolicy: {
 *         serviceEnabled: false,
 *     },
 * }, {
 *     dependsOn: [example],
 * });
 * ```
 *
 * ### Custom Control Configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const disabled = new aws.securityhub.ConfigurationPolicy("disabled", {
 *     name: "Custom Controls",
 *     description: "This is an example of configuration policy with custom control settings",
 *     configurationPolicy: {
 *         serviceEnabled: true,
 *         enabledStandardArns: [
 *             "arn:aws:securityhub:us-east-1::standards/aws-foundational-security-best-practices/v/1.0.0",
 *             "arn:aws:securityhub:::ruleset/cis-aws-foundations-benchmark/v/1.2.0",
 *         ],
 *         securityControlsConfiguration: {
 *             enabledControlIdentifiers: [
 *                 "APIGateway.1",
 *                 "IAM.7",
 *             ],
 *             securityControlCustomParameters: [
 *                 {
 *                     securityControlId: "APIGateway.1",
 *                     parameters: [{
 *                         name: "loggingLevel",
 *                         valueType: "CUSTOM",
 *                         "enum": {
 *                             value: "INFO",
 *                         },
 *                     }],
 *                 },
 *                 {
 *                     securityControlId: "IAM.7",
 *                     parameters: [
 *                         {
 *                             name: "RequireLowercaseCharacters",
 *                             valueType: "CUSTOM",
 *                             bool: {
 *                                 value: false,
 *                             },
 *                         },
 *                         {
 *                             name: "MaxPasswordAge",
 *                             valueType: "CUSTOM",
 *                             int: {
 *                                 value: 60,
 *                             },
 *                         },
 *                     ],
 *                 },
 *             ],
 *         },
 *     },
 * }, {
 *     dependsOn: [example],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import an existing Security Hub enabled account using the universally unique identifier (UUID) of the policy. For example:
 *
 * ```sh
 * $ pulumi import aws:securityhub/configurationPolicy:ConfigurationPolicy example "00000000-1111-2222-3333-444444444444"
 * ```
 */
export declare class ConfigurationPolicy extends pulumi.CustomResource {
    /**
     * Get an existing ConfigurationPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConfigurationPolicyState, opts?: pulumi.CustomResourceOptions): ConfigurationPolicy;
    /**
     * Returns true if the given object is an instance of ConfigurationPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ConfigurationPolicy;
    readonly arn: pulumi.Output<string>;
    /**
     * Defines how Security Hub is configured. See below.
     */
    readonly configurationPolicy: pulumi.Output<outputs.securityhub.ConfigurationPolicyConfigurationPolicy>;
    /**
     * The description of the configuration policy.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The name of the configuration policy.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a ConfigurationPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ConfigurationPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ConfigurationPolicy resources.
 */
export interface ConfigurationPolicyState {
    arn?: pulumi.Input<string>;
    /**
     * Defines how Security Hub is configured. See below.
     */
    configurationPolicy?: pulumi.Input<inputs.securityhub.ConfigurationPolicyConfigurationPolicy>;
    /**
     * The description of the configuration policy.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the configuration policy.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ConfigurationPolicy resource.
 */
export interface ConfigurationPolicyArgs {
    /**
     * Defines how Security Hub is configured. See below.
     */
    configurationPolicy: pulumi.Input<inputs.securityhub.ConfigurationPolicyConfigurationPolicy>;
    /**
     * The description of the configuration policy.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the configuration policy.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
