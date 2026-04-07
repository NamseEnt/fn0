import * as pulumi from "@pulumi/pulumi";
/**
 * Manages the default App Runner auto scaling configuration.
 * When creating or updating this resource the existing default auto scaling configuration will be set to non-default automatically.
 * When creating or updating this resource the configuration is automatically assigned as the default to the new services you create in the future. The new default designation doesn't affect the associations that were previously set for existing services.
 * Each account can have only one default auto scaling configuration per Region.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.apprunner.AutoScalingConfigurationVersion("example", {
 *     autoScalingConfigurationName: "example",
 *     maxConcurrency: 50,
 *     maxSize: 10,
 *     minSize: 2,
 * });
 * const exampleDefaultAutoScalingConfigurationVersion = new aws.apprunner.DefaultAutoScalingConfigurationVersion("example", {autoScalingConfigurationArn: example.arn});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import App Runner default auto scaling configurations using the current Region. For example:
 *
 * ```sh
 * $ pulumi import aws:apprunner/defaultAutoScalingConfigurationVersion:DefaultAutoScalingConfigurationVersion example us-west-2
 * ```
 */
export declare class DefaultAutoScalingConfigurationVersion extends pulumi.CustomResource {
    /**
     * Get an existing DefaultAutoScalingConfigurationVersion resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DefaultAutoScalingConfigurationVersionState, opts?: pulumi.CustomResourceOptions): DefaultAutoScalingConfigurationVersion;
    /**
     * Returns true if the given object is an instance of DefaultAutoScalingConfigurationVersion.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DefaultAutoScalingConfigurationVersion;
    /**
     * The ARN of the App Runner auto scaling configuration that you want to set as the default.
     */
    readonly autoScalingConfigurationArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a DefaultAutoScalingConfigurationVersion resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DefaultAutoScalingConfigurationVersionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DefaultAutoScalingConfigurationVersion resources.
 */
export interface DefaultAutoScalingConfigurationVersionState {
    /**
     * The ARN of the App Runner auto scaling configuration that you want to set as the default.
     */
    autoScalingConfigurationArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DefaultAutoScalingConfigurationVersion resource.
 */
export interface DefaultAutoScalingConfigurationVersionArgs {
    /**
     * The ARN of the App Runner auto scaling configuration that you want to set as the default.
     */
    autoScalingConfigurationArn: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
