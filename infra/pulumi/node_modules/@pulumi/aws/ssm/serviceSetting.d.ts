import * as pulumi from "@pulumi/pulumi";
/**
 * This setting defines how a user interacts with or uses a service or a feature of a service.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testSetting = new aws.ssm.ServiceSetting("test_setting", {
 *     settingId: "arn:aws:ssm:us-east-1:123456789012:servicesetting/ssm/parameter-store/high-throughput-enabled",
 *     settingValue: "true",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AWS SSM Service Setting using the `setting_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ssm/serviceSetting:ServiceSetting example arn:aws:ssm:us-east-1:123456789012:servicesetting/ssm/parameter-store/high-throughput-enabled
 * ```
 */
export declare class ServiceSetting extends pulumi.CustomResource {
    /**
     * Get an existing ServiceSetting resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ServiceSettingState, opts?: pulumi.CustomResourceOptions): ServiceSetting;
    /**
     * Returns true if the given object is an instance of ServiceSetting.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ServiceSetting;
    /**
     * ARN of the service setting.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ID of the service setting. Valid values are shown in the [AWS documentation](https://docs.aws.amazon.com/systems-manager/latest/APIReference/API_GetServiceSetting.html#API_GetServiceSetting_RequestSyntax).
     */
    readonly settingId: pulumi.Output<string>;
    /**
     * Value of the service setting.
     */
    readonly settingValue: pulumi.Output<string>;
    /**
     * Status of the service setting. Value can be `Default`, `Customized` or `PendingUpdate`.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Create a ServiceSetting resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ServiceSettingArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ServiceSetting resources.
 */
export interface ServiceSettingState {
    /**
     * ARN of the service setting.
     */
    arn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the service setting. Valid values are shown in the [AWS documentation](https://docs.aws.amazon.com/systems-manager/latest/APIReference/API_GetServiceSetting.html#API_GetServiceSetting_RequestSyntax).
     */
    settingId?: pulumi.Input<string>;
    /**
     * Value of the service setting.
     */
    settingValue?: pulumi.Input<string>;
    /**
     * Status of the service setting. Value can be `Default`, `Customized` or `PendingUpdate`.
     */
    status?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ServiceSetting resource.
 */
export interface ServiceSettingArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID of the service setting. Valid values are shown in the [AWS documentation](https://docs.aws.amazon.com/systems-manager/latest/APIReference/API_GetServiceSetting.html#API_GetServiceSetting_RequestSyntax).
     */
    settingId: pulumi.Input<string>;
    /**
     * Value of the service setting.
     */
    settingValue: pulumi.Input<string>;
}
