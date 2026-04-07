import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage AWS ECR Basic Scan Type
 *
 * ## Example Usage
 *
 * ### Configuring Basic Scanning
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const basicScanTypeVersion = new aws.ecr.AccountSetting("basic_scan_type_version", {
 *     name: "BASIC_SCAN_TYPE_VERSION",
 *     value: "AWS_NATIVE",
 * });
 * ```
 *
 * ### Configuring Registry Policy Scope
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const registryPolicyScope = new aws.ecr.AccountSetting("registry_policy_scope", {
 *     name: "REGISTRY_POLICY_SCOPE",
 *     value: "V2",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import EMR Security Configurations using the account setting name. For example:
 *
 * ```sh
 * $ pulumi import aws:ecr/accountSetting:AccountSetting foo BASIC_SCAN_TYPE_VERSION
 * ```
 */
export declare class AccountSetting extends pulumi.CustomResource {
    /**
     * Get an existing AccountSetting resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AccountSettingState, opts?: pulumi.CustomResourceOptions): AccountSetting;
    /**
     * Returns true if the given object is an instance of AccountSetting.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AccountSetting;
    /**
     * Name of the account setting. One of: `BASIC_SCAN_TYPE_VERSION`, `REGISTRY_POLICY_SCOPE`.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Setting value that is specified. Valid values are:
     * * If `name` is specified as `BASIC_SCAN_TYPE_VERSION`, one of: `AWS_NATIVE`, `CLAIR`.
     * * If `name` is specified as `REGISTRY_POLICY_SCOPE`, one of: `V1`, `V2`.
     */
    readonly value: pulumi.Output<string>;
    /**
     * Create a AccountSetting resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AccountSettingArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AccountSetting resources.
 */
export interface AccountSettingState {
    /**
     * Name of the account setting. One of: `BASIC_SCAN_TYPE_VERSION`, `REGISTRY_POLICY_SCOPE`.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Setting value that is specified. Valid values are:
     * * If `name` is specified as `BASIC_SCAN_TYPE_VERSION`, one of: `AWS_NATIVE`, `CLAIR`.
     * * If `name` is specified as `REGISTRY_POLICY_SCOPE`, one of: `V1`, `V2`.
     */
    value?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AccountSetting resource.
 */
export interface AccountSettingArgs {
    /**
     * Name of the account setting. One of: `BASIC_SCAN_TYPE_VERSION`, `REGISTRY_POLICY_SCOPE`.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Setting value that is specified. Valid values are:
     * * If `name` is specified as `BASIC_SCAN_TYPE_VERSION`, one of: `AWS_NATIVE`, `CLAIR`.
     * * If `name` is specified as `REGISTRY_POLICY_SCOPE`, one of: `V1`, `V2`.
     */
    value: pulumi.Input<string>;
}
