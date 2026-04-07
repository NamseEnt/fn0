import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Amazon Managed Grafana workspace resource.
 *
 * ## Example Usage
 *
 * ### Basic configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const assume = new aws.iam.Role("assume", {
 *     name: "grafana-assume",
 *     assumeRolePolicy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Action: "sts:AssumeRole",
 *             Effect: "Allow",
 *             Sid: "",
 *             Principal: {
 *                 Service: "grafana.amazonaws.com",
 *             },
 *         }],
 *     }),
 * });
 * const example = new aws.grafana.Workspace("example", {
 *     accountAccessType: "CURRENT_ACCOUNT",
 *     authenticationProviders: ["SAML"],
 *     permissionType: "SERVICE_MANAGED",
 *     roleArn: assume.arn,
 * });
 * ```
 *
 * ### Workspace configuration options
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.grafana.Workspace("example", {
 *     accountAccessType: "CURRENT_ACCOUNT",
 *     authenticationProviders: ["SAML"],
 *     permissionType: "SERVICE_MANAGED",
 *     roleArn: assume.arn,
 *     configuration: JSON.stringify({
 *         plugins: {
 *             pluginAdminEnabled: true,
 *         },
 *         unifiedAlerting: {
 *             enabled: false,
 *         },
 *     }),
 * });
 * ```
 *
 * The optional argument `configuration` is a JSON string that disables the unified `Grafana Alerting` (Grafana version 10 or newer) and enables `Plugin Management` (Grafana version 9 or newer) on the Grafana Workspaces.
 *
 * For more information about using Grafana alerting, and the effects of turning it on or off, see [Alerts in Grafana version 10](https://docs.aws.amazon.com/grafana/latest/userguide/v10-alerts.html).
 *
 * ## Import
 *
 * Using `pulumi import`, import Grafana Workspace using the workspace's `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:grafana/workspace:Workspace example g-2054c75a02
 * ```
 */
export declare class Workspace extends pulumi.CustomResource {
    /**
     * Get an existing Workspace resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: WorkspaceState, opts?: pulumi.CustomResourceOptions): Workspace;
    /**
     * Returns true if the given object is an instance of Workspace.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Workspace;
    /**
     * The type of account access for the workspace. Valid values are `CURRENT_ACCOUNT` and `ORGANIZATION`. If `ORGANIZATION` is specified, then `organizationalUnits` must also be present.
     */
    readonly accountAccessType: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the Grafana workspace.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The authentication providers for the workspace. Valid values are `AWS_SSO`, `SAML`, or both.
     */
    readonly authenticationProviders: pulumi.Output<string[]>;
    /**
     * The configuration string for the workspace that you create. For more information about the format and configuration options available, see [Working in your Grafana workspace](https://docs.aws.amazon.com/grafana/latest/userguide/AMG-configure-workspace.html).
     */
    readonly configuration: pulumi.Output<string>;
    /**
     * The data sources for the workspace. Valid values are `AMAZON_OPENSEARCH_SERVICE`, `ATHENA`, `CLOUDWATCH`, `PROMETHEUS`, `REDSHIFT`, `SITEWISE`, `TIMESTREAM`, `TWINMAKER`, XRAY`
     */
    readonly dataSources: pulumi.Output<string[] | undefined>;
    /**
     * The workspace description.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The endpoint of the Grafana workspace.
     */
    readonly endpoint: pulumi.Output<string>;
    /**
     * Specifies the version of Grafana to support in the new workspace. Supported values are `8.4`, `9.4` and `10.4`. If not specified, defaults to the latest version.
     */
    readonly grafanaVersion: pulumi.Output<string>;
    /**
     * The Grafana workspace name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Configuration for network access to your workspace.See Network Access Control below.
     */
    readonly networkAccessControl: pulumi.Output<outputs.grafana.WorkspaceNetworkAccessControl | undefined>;
    /**
     * The notification destinations. If a data source is specified here, Amazon Managed Grafana will create IAM roles and permissions needed to use these destinations. Must be set to `SNS`.
     */
    readonly notificationDestinations: pulumi.Output<string[] | undefined>;
    /**
     * The role name that the workspace uses to access resources through Amazon Organizations.
     */
    readonly organizationRoleName: pulumi.Output<string | undefined>;
    /**
     * The Amazon Organizations organizational units that the workspace is authorized to use data sources from.
     */
    readonly organizationalUnits: pulumi.Output<string[] | undefined>;
    /**
     * The permission type of the workspace. If `SERVICE_MANAGED` is specified, the IAM roles and IAM policy attachments are generated automatically. If `CUSTOMER_MANAGED` is specified, the IAM roles and IAM policy attachments will not be created.
     *
     * The following arguments are optional:
     */
    readonly permissionType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The IAM role ARN that the workspace assumes.
     */
    readonly roleArn: pulumi.Output<string | undefined>;
    readonly samlConfigurationStatus: pulumi.Output<string>;
    /**
     * The AWS CloudFormation stack set name that provisions IAM roles to be used by the workspace.
     */
    readonly stackSetName: pulumi.Output<string | undefined>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
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
    /**
     * The configuration settings for an Amazon VPC that contains data sources for your Grafana workspace to connect to. See VPC Configuration below.
     */
    readonly vpcConfiguration: pulumi.Output<outputs.grafana.WorkspaceVpcConfiguration | undefined>;
    /**
     * Create a Workspace resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: WorkspaceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Workspace resources.
 */
export interface WorkspaceState {
    /**
     * The type of account access for the workspace. Valid values are `CURRENT_ACCOUNT` and `ORGANIZATION`. If `ORGANIZATION` is specified, then `organizationalUnits` must also be present.
     */
    accountAccessType?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the Grafana workspace.
     */
    arn?: pulumi.Input<string>;
    /**
     * The authentication providers for the workspace. Valid values are `AWS_SSO`, `SAML`, or both.
     */
    authenticationProviders?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The configuration string for the workspace that you create. For more information about the format and configuration options available, see [Working in your Grafana workspace](https://docs.aws.amazon.com/grafana/latest/userguide/AMG-configure-workspace.html).
     */
    configuration?: pulumi.Input<string>;
    /**
     * The data sources for the workspace. Valid values are `AMAZON_OPENSEARCH_SERVICE`, `ATHENA`, `CLOUDWATCH`, `PROMETHEUS`, `REDSHIFT`, `SITEWISE`, `TIMESTREAM`, `TWINMAKER`, XRAY`
     */
    dataSources?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The workspace description.
     */
    description?: pulumi.Input<string>;
    /**
     * The endpoint of the Grafana workspace.
     */
    endpoint?: pulumi.Input<string>;
    /**
     * Specifies the version of Grafana to support in the new workspace. Supported values are `8.4`, `9.4` and `10.4`. If not specified, defaults to the latest version.
     */
    grafanaVersion?: pulumi.Input<string>;
    /**
     * The Grafana workspace name.
     */
    name?: pulumi.Input<string>;
    /**
     * Configuration for network access to your workspace.See Network Access Control below.
     */
    networkAccessControl?: pulumi.Input<inputs.grafana.WorkspaceNetworkAccessControl>;
    /**
     * The notification destinations. If a data source is specified here, Amazon Managed Grafana will create IAM roles and permissions needed to use these destinations. Must be set to `SNS`.
     */
    notificationDestinations?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The role name that the workspace uses to access resources through Amazon Organizations.
     */
    organizationRoleName?: pulumi.Input<string>;
    /**
     * The Amazon Organizations organizational units that the workspace is authorized to use data sources from.
     */
    organizationalUnits?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The permission type of the workspace. If `SERVICE_MANAGED` is specified, the IAM roles and IAM policy attachments are generated automatically. If `CUSTOMER_MANAGED` is specified, the IAM roles and IAM policy attachments will not be created.
     *
     * The following arguments are optional:
     */
    permissionType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IAM role ARN that the workspace assumes.
     */
    roleArn?: pulumi.Input<string>;
    samlConfigurationStatus?: pulumi.Input<string>;
    /**
     * The AWS CloudFormation stack set name that provisions IAM roles to be used by the workspace.
     */
    stackSetName?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
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
    /**
     * The configuration settings for an Amazon VPC that contains data sources for your Grafana workspace to connect to. See VPC Configuration below.
     */
    vpcConfiguration?: pulumi.Input<inputs.grafana.WorkspaceVpcConfiguration>;
}
/**
 * The set of arguments for constructing a Workspace resource.
 */
export interface WorkspaceArgs {
    /**
     * The type of account access for the workspace. Valid values are `CURRENT_ACCOUNT` and `ORGANIZATION`. If `ORGANIZATION` is specified, then `organizationalUnits` must also be present.
     */
    accountAccessType: pulumi.Input<string>;
    /**
     * The authentication providers for the workspace. Valid values are `AWS_SSO`, `SAML`, or both.
     */
    authenticationProviders: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The configuration string for the workspace that you create. For more information about the format and configuration options available, see [Working in your Grafana workspace](https://docs.aws.amazon.com/grafana/latest/userguide/AMG-configure-workspace.html).
     */
    configuration?: pulumi.Input<string>;
    /**
     * The data sources for the workspace. Valid values are `AMAZON_OPENSEARCH_SERVICE`, `ATHENA`, `CLOUDWATCH`, `PROMETHEUS`, `REDSHIFT`, `SITEWISE`, `TIMESTREAM`, `TWINMAKER`, XRAY`
     */
    dataSources?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The workspace description.
     */
    description?: pulumi.Input<string>;
    /**
     * Specifies the version of Grafana to support in the new workspace. Supported values are `8.4`, `9.4` and `10.4`. If not specified, defaults to the latest version.
     */
    grafanaVersion?: pulumi.Input<string>;
    /**
     * The Grafana workspace name.
     */
    name?: pulumi.Input<string>;
    /**
     * Configuration for network access to your workspace.See Network Access Control below.
     */
    networkAccessControl?: pulumi.Input<inputs.grafana.WorkspaceNetworkAccessControl>;
    /**
     * The notification destinations. If a data source is specified here, Amazon Managed Grafana will create IAM roles and permissions needed to use these destinations. Must be set to `SNS`.
     */
    notificationDestinations?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The role name that the workspace uses to access resources through Amazon Organizations.
     */
    organizationRoleName?: pulumi.Input<string>;
    /**
     * The Amazon Organizations organizational units that the workspace is authorized to use data sources from.
     */
    organizationalUnits?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The permission type of the workspace. If `SERVICE_MANAGED` is specified, the IAM roles and IAM policy attachments are generated automatically. If `CUSTOMER_MANAGED` is specified, the IAM roles and IAM policy attachments will not be created.
     *
     * The following arguments are optional:
     */
    permissionType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IAM role ARN that the workspace assumes.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * The AWS CloudFormation stack set name that provisions IAM roles to be used by the workspace.
     */
    stackSetName?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The configuration settings for an Amazon VPC that contains data sources for your Grafana workspace to connect to. See VPC Configuration below.
     */
    vpcConfiguration?: pulumi.Input<inputs.grafana.WorkspaceVpcConfiguration>;
}
