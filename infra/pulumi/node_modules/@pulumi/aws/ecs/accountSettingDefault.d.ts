import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an ECS default account setting for a specific ECS Resource name within a specific region. More information can be found on the [ECS Developer Guide](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/ecs-account-settings.html).
 *
 * > **NOTE:** The AWS API does not delete this resource. When you run `destroy`, the provider will attempt to disable the setting.
 *
 * > **NOTE:** Your AWS account may not support disabling `containerInstanceLongArnFormat`, `serviceLongArnFormat`, and `taskLongArnFormat`. If your account does not support disabling these, "destroying" this resource will not disable the setting nor cause a provider error. However, the AWS Provider will log an AWS error: `InvalidParameterException: You can no longer disable Long Arn settings`.
 *
 * ## Example Usage
 *
 * ### Enable the long task ARN format
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.ecs.AccountSettingDefault("test", {
 *     name: "taskLongArnFormat",
 *     value: "enabled",
 * });
 * ```
 *
 * ### Set the default log driver mode to non-blocking
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.ecs.AccountSettingDefault("test", {
 *     name: "defaultLogDriverMode",
 *     value: "non-blocking",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import ECS Account Setting defaults using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:ecs/accountSettingDefault:AccountSettingDefault example taskLongArnFormat
 * ```
 */
export declare class AccountSettingDefault extends pulumi.CustomResource {
    /**
     * Get an existing AccountSettingDefault resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AccountSettingDefaultState, opts?: pulumi.CustomResourceOptions): AccountSettingDefault;
    /**
     * Returns true if the given object is an instance of AccountSettingDefault.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AccountSettingDefault;
    /**
     * Name of the account setting to set.
     */
    readonly name: pulumi.Output<string>;
    readonly principalArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * State of the setting.
     */
    readonly value: pulumi.Output<string>;
    /**
     * Create a AccountSettingDefault resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AccountSettingDefaultArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AccountSettingDefault resources.
 */
export interface AccountSettingDefaultState {
    /**
     * Name of the account setting to set.
     */
    name?: pulumi.Input<string>;
    principalArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * State of the setting.
     */
    value?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AccountSettingDefault resource.
 */
export interface AccountSettingDefaultArgs {
    /**
     * Name of the account setting to set.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * State of the setting.
     */
    value: pulumi.Input<string>;
}
