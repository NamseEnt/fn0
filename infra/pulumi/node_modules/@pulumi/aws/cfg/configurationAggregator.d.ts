import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS Config Configuration Aggregator
 *
 * ## Example Usage
 *
 * ### Account Based Aggregation
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const account = new aws.cfg.ConfigurationAggregator("account", {
 *     name: "example",
 *     accountAggregationSource: {
 *         accountIds: ["123456789012"],
 *         regions: ["us-west-2"],
 *     },
 * });
 * ```
 *
 * ### Organization Based Aggregation
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const assumeRole = aws.iam.getPolicyDocument({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["config.amazonaws.com"],
 *         }],
 *         actions: ["sts:AssumeRole"],
 *     }],
 * });
 * const organizationRole = new aws.iam.Role("organization", {
 *     name: "example",
 *     assumeRolePolicy: assumeRole.then(assumeRole => assumeRole.json),
 * });
 * const organizationRolePolicyAttachment = new aws.iam.RolePolicyAttachment("organization", {
 *     role: organizationRole.name,
 *     policyArn: "arn:aws:iam::aws:policy/service-role/AWSConfigRoleForOrganizations",
 * });
 * const organization = new aws.cfg.ConfigurationAggregator("organization", {
 *     name: "example",
 *     organizationAggregationSource: {
 *         allRegions: true,
 *         roleArn: organizationRole.arn,
 *     },
 * }, {
 *     dependsOn: [organizationRolePolicyAttachment],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Configuration Aggregators using the name. For example:
 *
 * ```sh
 * $ pulumi import aws:cfg/configurationAggregator:ConfigurationAggregator example foo
 * ```
 */
export declare class ConfigurationAggregator extends pulumi.CustomResource {
    /**
     * Get an existing ConfigurationAggregator resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConfigurationAggregatorState, opts?: pulumi.CustomResourceOptions): ConfigurationAggregator;
    /**
     * Returns true if the given object is an instance of ConfigurationAggregator.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ConfigurationAggregator;
    /**
     * The account(s) to aggregate config data from as documented below.
     */
    readonly accountAggregationSource: pulumi.Output<outputs.cfg.ConfigurationAggregatorAccountAggregationSource | undefined>;
    /**
     * The ARN of the aggregator
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The name of the configuration aggregator.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The organization to aggregate config data from as documented below.
     */
    readonly organizationAggregationSource: pulumi.Output<outputs.cfg.ConfigurationAggregatorOrganizationAggregationSource | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * Either `accountAggregationSource` or `organizationAggregationSource` must be specified.
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
     * Create a ConfigurationAggregator resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ConfigurationAggregatorArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ConfigurationAggregator resources.
 */
export interface ConfigurationAggregatorState {
    /**
     * The account(s) to aggregate config data from as documented below.
     */
    accountAggregationSource?: pulumi.Input<inputs.cfg.ConfigurationAggregatorAccountAggregationSource>;
    /**
     * The ARN of the aggregator
     */
    arn?: pulumi.Input<string>;
    /**
     * The name of the configuration aggregator.
     */
    name?: pulumi.Input<string>;
    /**
     * The organization to aggregate config data from as documented below.
     */
    organizationAggregationSource?: pulumi.Input<inputs.cfg.ConfigurationAggregatorOrganizationAggregationSource>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * Either `accountAggregationSource` or `organizationAggregationSource` must be specified.
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
 * The set of arguments for constructing a ConfigurationAggregator resource.
 */
export interface ConfigurationAggregatorArgs {
    /**
     * The account(s) to aggregate config data from as documented below.
     */
    accountAggregationSource?: pulumi.Input<inputs.cfg.ConfigurationAggregatorAccountAggregationSource>;
    /**
     * The name of the configuration aggregator.
     */
    name?: pulumi.Input<string>;
    /**
     * The organization to aggregate config data from as documented below.
     */
    organizationAggregationSource?: pulumi.Input<inputs.cfg.ConfigurationAggregatorOrganizationAggregationSource>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     *
     * Either `accountAggregationSource` or `organizationAggregationSource` must be specified.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
