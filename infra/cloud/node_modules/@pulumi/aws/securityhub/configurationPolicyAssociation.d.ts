import * as pulumi from "@pulumi/pulumi";
/**
 * Manages Security Hub configuration policy associations.
 *
 * > **NOTE:** This resource requires `aws.securityhub.OrganizationConfiguration` to be configured with type `CENTRAL`. More information about Security Hub central configuration and configuration policies can be found in the [How Security Hub configuration policies work](https://docs.aws.amazon.com/securityhub/latest/userguide/configuration-policies-overview.html) documentation.
 *
 * ## Example Usage
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
 * const accountExample = new aws.securityhub.ConfigurationPolicyAssociation("account_example", {
 *     targetId: "123456789012",
 *     policyId: exampleConfigurationPolicy.id,
 * });
 * const rootExample = new aws.securityhub.ConfigurationPolicyAssociation("root_example", {
 *     targetId: "r-abcd",
 *     policyId: exampleConfigurationPolicy.id,
 * });
 * const ouExample = new aws.securityhub.ConfigurationPolicyAssociation("ou_example", {
 *     targetId: "ou-abcd-12345678",
 *     policyId: exampleConfigurationPolicy.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import an existing Security Hub enabled account using the target id. For example:
 *
 * ```sh
 * $ pulumi import aws:securityhub/configurationPolicyAssociation:ConfigurationPolicyAssociation example_account_association 123456789012
 * ```
 */
export declare class ConfigurationPolicyAssociation extends pulumi.CustomResource {
    /**
     * Get an existing ConfigurationPolicyAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConfigurationPolicyAssociationState, opts?: pulumi.CustomResourceOptions): ConfigurationPolicyAssociation;
    /**
     * Returns true if the given object is an instance of ConfigurationPolicyAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ConfigurationPolicyAssociation;
    /**
     * The universally unique identifier (UUID) of the configuration policy.
     */
    readonly policyId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The identifier of the target account, organizational unit, or the root to associate with the specified configuration.
     */
    readonly targetId: pulumi.Output<string>;
    /**
     * Create a ConfigurationPolicyAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ConfigurationPolicyAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ConfigurationPolicyAssociation resources.
 */
export interface ConfigurationPolicyAssociationState {
    /**
     * The universally unique identifier (UUID) of the configuration policy.
     */
    policyId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The identifier of the target account, organizational unit, or the root to associate with the specified configuration.
     */
    targetId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ConfigurationPolicyAssociation resource.
 */
export interface ConfigurationPolicyAssociationArgs {
    /**
     * The universally unique identifier (UUID) of the configuration policy.
     */
    policyId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The identifier of the target account, organizational unit, or the root to associate with the specified configuration.
     */
    targetId: pulumi.Input<string>;
}
