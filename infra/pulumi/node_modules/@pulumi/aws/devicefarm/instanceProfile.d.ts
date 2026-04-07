import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage AWS Device Farm Instance Profiles.
 * ∂
 * > **NOTE:** AWS currently has limited regional support for Device Farm (e.g., `us-west-2`). See [AWS Device Farm endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/devicefarm.html) for information on supported regions.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.devicefarm.InstanceProfile("example", {name: "example"});
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Device Farm instance profile.
 *
 * Using `pulumi import`, import DeviceFarm Instance Profiles using their ARN. For example:
 *
 * console
 *
 * % pulumi import aws_devicefarm_instance_profile.example arn:aws:devicefarm:us-west-2:123456789012:instanceprofile:4fa784c7-ccb4-4dbf-ba4f-02198320daa1
 */
export declare class InstanceProfile extends pulumi.CustomResource {
    /**
     * Get an existing InstanceProfile resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InstanceProfileState, opts?: pulumi.CustomResourceOptions): InstanceProfile;
    /**
     * Returns true if the given object is an instance of InstanceProfile.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is InstanceProfile;
    /**
     * The Amazon Resource Name of this instance profile.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The description of the instance profile.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * An array of strings that specifies the list of app packages that should not be cleaned up from the device after a test run.
     */
    readonly excludeAppPackagesFromCleanups: pulumi.Output<string[] | undefined>;
    /**
     * The name for the instance profile.
     */
    readonly name: pulumi.Output<string>;
    /**
     * When set to `true`, Device Farm removes app packages after a test run. The default value is `false` for private devices.
     */
    readonly packageCleanup: pulumi.Output<boolean | undefined>;
    /**
     * When set to `true`, Device Farm reboots the instance after a test run. The default value is `true`.
     */
    readonly rebootAfterUse: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a InstanceProfile resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: InstanceProfileArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering InstanceProfile resources.
 */
export interface InstanceProfileState {
    /**
     * The Amazon Resource Name of this instance profile.
     */
    arn?: pulumi.Input<string>;
    /**
     * The description of the instance profile.
     */
    description?: pulumi.Input<string>;
    /**
     * An array of strings that specifies the list of app packages that should not be cleaned up from the device after a test run.
     */
    excludeAppPackagesFromCleanups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name for the instance profile.
     */
    name?: pulumi.Input<string>;
    /**
     * When set to `true`, Device Farm removes app packages after a test run. The default value is `false` for private devices.
     */
    packageCleanup?: pulumi.Input<boolean>;
    /**
     * When set to `true`, Device Farm reboots the instance after a test run. The default value is `true`.
     */
    rebootAfterUse?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a InstanceProfile resource.
 */
export interface InstanceProfileArgs {
    /**
     * The description of the instance profile.
     */
    description?: pulumi.Input<string>;
    /**
     * An array of strings that specifies the list of app packages that should not be cleaned up from the device after a test run.
     */
    excludeAppPackagesFromCleanups?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name for the instance profile.
     */
    name?: pulumi.Input<string>;
    /**
     * When set to `true`, Device Farm removes app packages after a test run. The default value is `false` for private devices.
     */
    packageCleanup?: pulumi.Input<boolean>;
    /**
     * When set to `true`, Device Farm reboots the instance after a test run. The default value is `true`.
     */
    rebootAfterUse?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
