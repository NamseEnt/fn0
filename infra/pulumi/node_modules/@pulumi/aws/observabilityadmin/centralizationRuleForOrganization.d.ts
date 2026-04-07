import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an AWS CloudWatch Observability Admin Centralization Rule For Organization.
 *
 * Centralization rules enable you to centralize log data from multiple AWS accounts and regions within your organization to a single destination account and region. This helps with log management, compliance, and cost optimization by consolidating logs in a central location.
 *
 * This requires an AWS account within an organization with at least [delegated administrator permissions](https://docs.aws.amazon.com/organizations/latest/APIReference/API_RegisterDelegatedAdministrator.html).
 *
 * ## Example Usage
 *
 * ### Basic Centralization Rule
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const currentGetOrganization = aws.organizations.getOrganization({});
 * const example = new aws.observabilityadmin.CentralizationRuleForOrganization("example", {
 *     ruleName: "example-centralization-rule",
 *     rule: {
 *         destination: {
 *             region: "eu-west-1",
 *             account: current.then(current => current.accountId),
 *         },
 *         source: {
 *             regions: ["ap-southeast-1"],
 *             scope: currentGetOrganization.then(currentGetOrganization => `OrganizationId = '${currentGetOrganization.id}'`),
 *             sourceLogsConfiguration: {
 *                 encryptedLogGroupStrategy: "SKIP",
 *                 logGroupSelectionCriteria: "*",
 *             },
 *         },
 *     },
 *     tags: {
 *         Name: "example-centralization-rule",
 *         Environment: "production",
 *     },
 * });
 * ```
 *
 * ### Advanced Configuration with Encryption and Backup
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const currentGetOrganization = aws.organizations.getOrganization({});
 * const advanced = new aws.observabilityadmin.CentralizationRuleForOrganization("advanced", {
 *     ruleName: "advanced-centralization-rule",
 *     rule: {
 *         destination: {
 *             region: "eu-west-1",
 *             account: current.then(current => current.accountId),
 *             destinationLogsConfiguration: {
 *                 logsEncryptionConfiguration: {
 *                     encryptionStrategy: "AWS_OWNED",
 *                 },
 *                 backupConfiguration: {
 *                     region: "us-west-1",
 *                 },
 *             },
 *         },
 *         source: {
 *             regions: [
 *                 "ap-southeast-1",
 *                 "us-east-1",
 *             ],
 *             scope: currentGetOrganization.then(currentGetOrganization => `OrganizationId = '${currentGetOrganization.id}'`),
 *             sourceLogsConfiguration: {
 *                 encryptedLogGroupStrategy: "ALLOW",
 *                 logGroupSelectionCriteria: "*",
 *             },
 *         },
 *     },
 *     tags: {
 *         Name: "advanced-centralization-rule",
 *         Environment: "production",
 *         Team: "observability",
 *     },
 * });
 * ```
 *
 * ### Selective Log Group Filtering
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const currentGetOrganization = aws.organizations.getOrganization({});
 * const filtered = new aws.observabilityadmin.CentralizationRuleForOrganization("filtered", {
 *     ruleName: "filtered-centralization-rule",
 *     rule: {
 *         destination: {
 *             region: "eu-west-1",
 *             account: current.then(current => current.accountId),
 *         },
 *         source: {
 *             regions: [
 *                 "ap-southeast-1",
 *                 "us-east-1",
 *             ],
 *             scope: currentGetOrganization.then(currentGetOrganization => `OrganizationId = '${currentGetOrganization.id}'`),
 *             sourceLogsConfiguration: {
 *                 encryptedLogGroupStrategy: "ALLOW",
 *                 logGroupSelectionCriteria: "LogGroupName LIKE '/aws/lambda%'",
 *             },
 *         },
 *     },
 *     tags: {
 *         Name: "filtered-centralization-rule",
 *         Filter: "lambda-logs",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CloudWatch Observability Admin Centralization Rule For Organization using the `rule_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:observabilityadmin/centralizationRuleForOrganization:CentralizationRuleForOrganization example example-centralization-rule
 * ```
 */
export declare class CentralizationRuleForOrganization extends pulumi.CustomResource {
    /**
     * Get an existing CentralizationRuleForOrganization resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CentralizationRuleForOrganizationState, opts?: pulumi.CustomResourceOptions): CentralizationRuleForOrganization;
    /**
     * Returns true if the given object is an instance of CentralizationRuleForOrganization.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CentralizationRuleForOrganization;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Configuration block for the centralization rule. See `rule` below.
     *
     * The following arguments are optional:
     */
    readonly rule: pulumi.Output<outputs.observabilityadmin.CentralizationRuleForOrganizationRule | undefined>;
    /**
     * ARN of the centralization rule.
     */
    readonly ruleArn: pulumi.Output<string>;
    /**
     * Name of the centralization rule. Must be unique within the organization.
     */
    readonly ruleName: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.observabilityadmin.CentralizationRuleForOrganizationTimeouts | undefined>;
    /**
     * Create a CentralizationRuleForOrganization resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CentralizationRuleForOrganizationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CentralizationRuleForOrganization resources.
 */
export interface CentralizationRuleForOrganizationState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for the centralization rule. See `rule` below.
     *
     * The following arguments are optional:
     */
    rule?: pulumi.Input<inputs.observabilityadmin.CentralizationRuleForOrganizationRule>;
    /**
     * ARN of the centralization rule.
     */
    ruleArn?: pulumi.Input<string>;
    /**
     * Name of the centralization rule. Must be unique within the organization.
     */
    ruleName?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.observabilityadmin.CentralizationRuleForOrganizationTimeouts>;
}
/**
 * The set of arguments for constructing a CentralizationRuleForOrganization resource.
 */
export interface CentralizationRuleForOrganizationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block for the centralization rule. See `rule` below.
     *
     * The following arguments are optional:
     */
    rule?: pulumi.Input<inputs.observabilityadmin.CentralizationRuleForOrganizationRule>;
    /**
     * Name of the centralization rule. Must be unique within the organization.
     */
    ruleName: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.observabilityadmin.CentralizationRuleForOrganizationTimeouts>;
}
