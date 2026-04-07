import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage the AWS Config retention configuration.
 * The retention configuration defines the number of days that AWS Config stores historical information.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cfg.RetentionConfiguration("example", {retentionPeriodInDays: 90});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import the AWS Config retention configuration using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:cfg/retentionConfiguration:RetentionConfiguration example default
 * ```
 */
export declare class RetentionConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing RetentionConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RetentionConfigurationState, opts?: pulumi.CustomResourceOptions): RetentionConfiguration;
    /**
     * Returns true if the given object is an instance of RetentionConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RetentionConfiguration;
    /**
     * The name of the retention configuration object. The object is always named **default**.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The number of days AWS Config stores historical information.
     */
    readonly retentionPeriodInDays: pulumi.Output<number>;
    /**
     * Create a RetentionConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RetentionConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RetentionConfiguration resources.
 */
export interface RetentionConfigurationState {
    /**
     * The name of the retention configuration object. The object is always named **default**.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The number of days AWS Config stores historical information.
     */
    retentionPeriodInDays?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a RetentionConfiguration resource.
 */
export interface RetentionConfigurationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The number of days AWS Config stores historical information.
     */
    retentionPeriodInDays: pulumi.Input<number>;
}
