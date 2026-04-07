import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a resource to manage whether serial console access is enabled for your AWS account in the current AWS region.
 *
 * > **NOTE:** Removing this resource disables serial console access.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.SerialConsoleAccess("example", {enabled: true});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import serial console access state. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/serialConsoleAccess:SerialConsoleAccess example default
 * ```
 */
export declare class SerialConsoleAccess extends pulumi.CustomResource {
    /**
     * Get an existing SerialConsoleAccess resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: SerialConsoleAccessState, opts?: pulumi.CustomResourceOptions): SerialConsoleAccess;
    /**
     * Returns true if the given object is an instance of SerialConsoleAccess.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is SerialConsoleAccess;
    /**
     * Whether or not serial console access is enabled. Valid values are `true` or `false`. Defaults to `true`.
     */
    readonly enabled: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a SerialConsoleAccess resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: SerialConsoleAccessArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering SerialConsoleAccess resources.
 */
export interface SerialConsoleAccessState {
    /**
     * Whether or not serial console access is enabled. Valid values are `true` or `false`. Defaults to `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a SerialConsoleAccess resource.
 */
export interface SerialConsoleAccessArgs {
    /**
     * Whether or not serial console access is enabled. Valid values are `true` or `false`. Defaults to `true`.
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
