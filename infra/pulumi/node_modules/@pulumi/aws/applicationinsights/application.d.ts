import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a ApplicationInsights Application resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleGroup = new aws.resourcegroups.Group("example", {
 *     name: "example",
 *     resourceQuery: {
 *         query: JSON.stringify({
 *             ResourceTypeFilters: ["AWS::EC2::Instance"],
 *             TagFilters: [{
 *                 Key: "Stage",
 *                 Values: ["Test"],
 *             }],
 *         }),
 *     },
 * });
 * const example = new aws.applicationinsights.Application("example", {resourceGroupName: exampleGroup.name});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import ApplicationInsights Applications using the `resource_group_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:applicationinsights/application:Application some some-application
 * ```
 */
export declare class Application extends pulumi.CustomResource {
    /**
     * Get an existing Application resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ApplicationState, opts?: pulumi.CustomResourceOptions): Application;
    /**
     * Returns true if the given object is an instance of Application.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Application;
    /**
     * ARN of the Application.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Indicates whether Application Insights automatically configures unmonitored resources in the resource group.
     */
    readonly autoConfigEnabled: pulumi.Output<boolean | undefined>;
    /**
     * Configures all of the resources in the resource group by applying the recommended configurations.
     */
    readonly autoCreate: pulumi.Output<boolean | undefined>;
    /**
     * Indicates whether Application Insights can listen to CloudWatch events for the application resources, such as instance terminated, failed deployment, and others.
     */
    readonly cweMonitorEnabled: pulumi.Output<boolean | undefined>;
    /**
     * Application Insights can create applications based on a resource group or on an account. To create an account-based application using all of the resources in the account, set this parameter to `ACCOUNT_BASED`.
     */
    readonly groupingType: pulumi.Output<string | undefined>;
    /**
     * When set to `true`, creates opsItems for any problems detected on an application.
     */
    readonly opsCenterEnabled: pulumi.Output<boolean | undefined>;
    /**
     * SNS topic provided to Application Insights that is associated to the created opsItem. Allows you to receive notifications for updates to the opsItem.
     */
    readonly opsItemSnsTopicArn: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name of the resource group.
     *
     * The following arguments are optional:
     */
    readonly resourceGroupName: pulumi.Output<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Application resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ApplicationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Application resources.
 */
export interface ApplicationState {
    /**
     * ARN of the Application.
     */
    arn?: pulumi.Input<string>;
    /**
     * Indicates whether Application Insights automatically configures unmonitored resources in the resource group.
     */
    autoConfigEnabled?: pulumi.Input<boolean>;
    /**
     * Configures all of the resources in the resource group by applying the recommended configurations.
     */
    autoCreate?: pulumi.Input<boolean>;
    /**
     * Indicates whether Application Insights can listen to CloudWatch events for the application resources, such as instance terminated, failed deployment, and others.
     */
    cweMonitorEnabled?: pulumi.Input<boolean>;
    /**
     * Application Insights can create applications based on a resource group or on an account. To create an account-based application using all of the resources in the account, set this parameter to `ACCOUNT_BASED`.
     */
    groupingType?: pulumi.Input<string>;
    /**
     * When set to `true`, creates opsItems for any problems detected on an application.
     */
    opsCenterEnabled?: pulumi.Input<boolean>;
    /**
     * SNS topic provided to Application Insights that is associated to the created opsItem. Allows you to receive notifications for updates to the opsItem.
     */
    opsItemSnsTopicArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the resource group.
     *
     * The following arguments are optional:
     */
    resourceGroupName?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
}
/**
 * The set of arguments for constructing a Application resource.
 */
export interface ApplicationArgs {
    /**
     * Indicates whether Application Insights automatically configures unmonitored resources in the resource group.
     */
    autoConfigEnabled?: pulumi.Input<boolean>;
    /**
     * Configures all of the resources in the resource group by applying the recommended configurations.
     */
    autoCreate?: pulumi.Input<boolean>;
    /**
     * Indicates whether Application Insights can listen to CloudWatch events for the application resources, such as instance terminated, failed deployment, and others.
     */
    cweMonitorEnabled?: pulumi.Input<boolean>;
    /**
     * Application Insights can create applications based on a resource group or on an account. To create an account-based application using all of the resources in the account, set this parameter to `ACCOUNT_BASED`.
     */
    groupingType?: pulumi.Input<string>;
    /**
     * When set to `true`, creates opsItems for any problems detected on an application.
     */
    opsCenterEnabled?: pulumi.Input<boolean>;
    /**
     * SNS topic provided to Application Insights that is associated to the created opsItem. Allows you to receive notifications for updates to the opsItem.
     */
    opsItemSnsTopicArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the resource group.
     *
     * The following arguments are optional:
     */
    resourceGroupName: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
