import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AppStream fleet.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testFleet = new aws.appstream.Fleet("test_fleet", {
 *     name: "test-fleet",
 *     computeCapacity: {
 *         desiredInstances: 1,
 *     },
 *     description: "test fleet",
 *     idleDisconnectTimeoutInSeconds: 60,
 *     displayName: "test-fleet",
 *     enableDefaultInternetAccess: false,
 *     fleetType: "ON_DEMAND",
 *     imageName: "Amazon-AppStream2-Sample-Image-03-11-2023",
 *     instanceType: "stream.standard.large",
 *     maxUserDurationInSeconds: 600,
 *     vpcConfig: {
 *         subnetIds: ["subnet-06e9b13400c225127"],
 *     },
 *     tags: {
 *         TagName: "tag-value",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_appstream_fleet` using the id. For example:
 *
 * ```sh
 * $ pulumi import aws:appstream/fleet:Fleet example fleetNameExample
 * ```
 */
export declare class Fleet extends pulumi.CustomResource {
    /**
     * Get an existing Fleet resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: FleetState, opts?: pulumi.CustomResourceOptions): Fleet;
    /**
     * Returns true if the given object is an instance of Fleet.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Fleet;
    /**
     * ARN of the appstream fleet.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Configuration block for the desired capacity of the fleet. See below.
     */
    readonly computeCapacity: pulumi.Output<outputs.appstream.FleetComputeCapacity>;
    /**
     * Date and time, in UTC and extended RFC 3339 format, when the fleet was created.
     */
    readonly createdTime: pulumi.Output<string>;
    /**
     * Description to display.
     */
    readonly description: pulumi.Output<string>;
    /**
     * Amount of time that a streaming session remains active after users disconnect.
     */
    readonly disconnectTimeoutInSeconds: pulumi.Output<number>;
    /**
     * Human-readable friendly name for the AppStream fleet.
     */
    readonly displayName: pulumi.Output<string>;
    /**
     * Configuration block for the name of the directory and organizational unit (OU) to use to join the fleet to a Microsoft Active Directory domain. See below.
     */
    readonly domainJoinInfo: pulumi.Output<outputs.appstream.FleetDomainJoinInfo>;
    /**
     * Enables or disables default internet access for the fleet.
     */
    readonly enableDefaultInternetAccess: pulumi.Output<boolean>;
    /**
     * Fleet type. Valid values are: `ON_DEMAND`, `ALWAYS_ON`
     */
    readonly fleetType: pulumi.Output<string>;
    /**
     * ARN of the IAM role to apply to the fleet.
     */
    readonly iamRoleArn: pulumi.Output<string>;
    /**
     * Amount of time that users can be idle (inactive) before they are disconnected from their streaming session and the `disconnectTimeoutInSeconds` time interval begins. Defaults to `0`. Valid value is between `60` and `3600 `seconds.
     */
    readonly idleDisconnectTimeoutInSeconds: pulumi.Output<number | undefined>;
    /**
     * ARN of the public, private, or shared image to use.
     */
    readonly imageArn: pulumi.Output<string>;
    /**
     * Name of the image used to create the fleet.
     */
    readonly imageName: pulumi.Output<string>;
    /**
     * Instance type to use when launching fleet instances.
     */
    readonly instanceType: pulumi.Output<string>;
    /**
     * The maximum number of user sessions on an instance. This only applies to multi-session fleets.
     */
    readonly maxSessionsPerInstance: pulumi.Output<number | undefined>;
    /**
     * Maximum amount of time that a streaming session can remain active, in seconds.
     */
    readonly maxUserDurationInSeconds: pulumi.Output<number>;
    /**
     * Unique name for the fleet.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * State of the fleet. Can be `STARTING`, `RUNNING`, `STOPPING` or `STOPPED`
     */
    readonly state: pulumi.Output<string>;
    /**
     * AppStream 2.0 view that is displayed to your users when they stream from the fleet. When `APP` is specified, only the windows of applications opened by users display. When `DESKTOP` is specified, the standard desktop that is provided by the operating system displays. If not specified, defaults to `APP`.
     */
    readonly streamView: pulumi.Output<string>;
    /**
     * Map of tags to attach to AppStream instances.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Configuration block for the VPC configuration for the image builder. See below.
     */
    readonly vpcConfig: pulumi.Output<outputs.appstream.FleetVpcConfig>;
    /**
     * Create a Fleet resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: FleetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Fleet resources.
 */
export interface FleetState {
    /**
     * ARN of the appstream fleet.
     */
    arn?: pulumi.Input<string>;
    /**
     * Configuration block for the desired capacity of the fleet. See below.
     */
    computeCapacity?: pulumi.Input<inputs.appstream.FleetComputeCapacity>;
    /**
     * Date and time, in UTC and extended RFC 3339 format, when the fleet was created.
     */
    createdTime?: pulumi.Input<string>;
    /**
     * Description to display.
     */
    description?: pulumi.Input<string>;
    /**
     * Amount of time that a streaming session remains active after users disconnect.
     */
    disconnectTimeoutInSeconds?: pulumi.Input<number>;
    /**
     * Human-readable friendly name for the AppStream fleet.
     */
    displayName?: pulumi.Input<string>;
    /**
     * Configuration block for the name of the directory and organizational unit (OU) to use to join the fleet to a Microsoft Active Directory domain. See below.
     */
    domainJoinInfo?: pulumi.Input<inputs.appstream.FleetDomainJoinInfo>;
    /**
     * Enables or disables default internet access for the fleet.
     */
    enableDefaultInternetAccess?: pulumi.Input<boolean>;
    /**
     * Fleet type. Valid values are: `ON_DEMAND`, `ALWAYS_ON`
     */
    fleetType?: pulumi.Input<string>;
    /**
     * ARN of the IAM role to apply to the fleet.
     */
    iamRoleArn?: pulumi.Input<string>;
    /**
     * Amount of time that users can be idle (inactive) before they are disconnected from their streaming session and the `disconnectTimeoutInSeconds` time interval begins. Defaults to `0`. Valid value is between `60` and `3600 `seconds.
     */
    idleDisconnectTimeoutInSeconds?: pulumi.Input<number>;
    /**
     * ARN of the public, private, or shared image to use.
     */
    imageArn?: pulumi.Input<string>;
    /**
     * Name of the image used to create the fleet.
     */
    imageName?: pulumi.Input<string>;
    /**
     * Instance type to use when launching fleet instances.
     */
    instanceType?: pulumi.Input<string>;
    /**
     * The maximum number of user sessions on an instance. This only applies to multi-session fleets.
     */
    maxSessionsPerInstance?: pulumi.Input<number>;
    /**
     * Maximum amount of time that a streaming session can remain active, in seconds.
     */
    maxUserDurationInSeconds?: pulumi.Input<number>;
    /**
     * Unique name for the fleet.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * State of the fleet. Can be `STARTING`, `RUNNING`, `STOPPING` or `STOPPED`
     */
    state?: pulumi.Input<string>;
    /**
     * AppStream 2.0 view that is displayed to your users when they stream from the fleet. When `APP` is specified, only the windows of applications opened by users display. When `DESKTOP` is specified, the standard desktop that is provided by the operating system displays. If not specified, defaults to `APP`.
     */
    streamView?: pulumi.Input<string>;
    /**
     * Map of tags to attach to AppStream instances.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block for the VPC configuration for the image builder. See below.
     */
    vpcConfig?: pulumi.Input<inputs.appstream.FleetVpcConfig>;
}
/**
 * The set of arguments for constructing a Fleet resource.
 */
export interface FleetArgs {
    /**
     * Configuration block for the desired capacity of the fleet. See below.
     */
    computeCapacity: pulumi.Input<inputs.appstream.FleetComputeCapacity>;
    /**
     * Description to display.
     */
    description?: pulumi.Input<string>;
    /**
     * Amount of time that a streaming session remains active after users disconnect.
     */
    disconnectTimeoutInSeconds?: pulumi.Input<number>;
    /**
     * Human-readable friendly name for the AppStream fleet.
     */
    displayName?: pulumi.Input<string>;
    /**
     * Configuration block for the name of the directory and organizational unit (OU) to use to join the fleet to a Microsoft Active Directory domain. See below.
     */
    domainJoinInfo?: pulumi.Input<inputs.appstream.FleetDomainJoinInfo>;
    /**
     * Enables or disables default internet access for the fleet.
     */
    enableDefaultInternetAccess?: pulumi.Input<boolean>;
    /**
     * Fleet type. Valid values are: `ON_DEMAND`, `ALWAYS_ON`
     */
    fleetType?: pulumi.Input<string>;
    /**
     * ARN of the IAM role to apply to the fleet.
     */
    iamRoleArn?: pulumi.Input<string>;
    /**
     * Amount of time that users can be idle (inactive) before they are disconnected from their streaming session and the `disconnectTimeoutInSeconds` time interval begins. Defaults to `0`. Valid value is between `60` and `3600 `seconds.
     */
    idleDisconnectTimeoutInSeconds?: pulumi.Input<number>;
    /**
     * ARN of the public, private, or shared image to use.
     */
    imageArn?: pulumi.Input<string>;
    /**
     * Name of the image used to create the fleet.
     */
    imageName?: pulumi.Input<string>;
    /**
     * Instance type to use when launching fleet instances.
     */
    instanceType: pulumi.Input<string>;
    /**
     * The maximum number of user sessions on an instance. This only applies to multi-session fleets.
     */
    maxSessionsPerInstance?: pulumi.Input<number>;
    /**
     * Maximum amount of time that a streaming session can remain active, in seconds.
     */
    maxUserDurationInSeconds?: pulumi.Input<number>;
    /**
     * Unique name for the fleet.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * AppStream 2.0 view that is displayed to your users when they stream from the fleet. When `APP` is specified, only the windows of applications opened by users display. When `DESKTOP` is specified, the standard desktop that is provided by the operating system displays. If not specified, defaults to `APP`.
     */
    streamView?: pulumi.Input<string>;
    /**
     * Map of tags to attach to AppStream instances.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block for the VPC configuration for the image builder. See below.
     */
    vpcConfig?: pulumi.Input<inputs.appstream.FleetVpcConfig>;
}
