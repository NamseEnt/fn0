import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an [AWS Mainframe Modernization Environment](https://docs.aws.amazon.com/m2/latest/userguide/environments-m2.html).
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.m2.Environment("test", {
 *     name: "test-env",
 *     engineType: "bluage",
 *     instanceType: "M2.m5.large",
 *     securityGroups: ["sg-01234567890abcdef"],
 *     subnetIds: [
 *         "subnet-01234567890abcdef",
 *         "subnet-01234567890abcdea",
 *     ],
 * });
 * ```
 *
 * ### High Availability
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.m2.Environment("test", {
 *     name: "test-env",
 *     engineType: "bluage",
 *     instanceType: "M2.m5.large",
 *     securityGroups: ["sg-01234567890abcdef"],
 *     subnetIds: [
 *         "subnet-01234567890abcdef",
 *         "subnet-01234567890abcdea",
 *     ],
 *     highAvailabilityConfig: {
 *         desiredCapacity: 2,
 *     },
 * });
 * ```
 *
 * ### EFS Filesystem
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.m2.Environment("test", {
 *     name: "test-env",
 *     engineType: "bluage",
 *     instanceType: "M2.m5.large",
 *     securityGroups: ["sg-01234567890abcdef"],
 *     subnetIds: [
 *         "subnet-01234567890abcdef",
 *         "subnet-01234567890abcdea",
 *     ],
 *     storageConfiguration: {
 *         efs: {
 *             fileSystemId: "fs-01234567890abcdef",
 *             mountPoint: "/m2/mount/example",
 *         },
 *     },
 * });
 * ```
 *
 * ### FSX Filesystem
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.m2.Environment("test", {
 *     name: "test-env",
 *     engineType: "bluage",
 *     instanceType: "M2.m5.large",
 *     securityGroups: ["sg-01234567890abcdef"],
 *     subnetIds: [
 *         "subnet-01234567890abcdef",
 *         "subnet-01234567890abcdea",
 *     ],
 *     storageConfiguration: {
 *         fsx: {
 *             fileSystemId: "fs-01234567890abcdef",
 *             mountPoint: "/m2/mount/example",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Mainframe Modernization Environment using the `01234567890abcdef012345678`. For example:
 *
 * ```sh
 * $ pulumi import aws:m2/environment:Environment example 01234567890abcdef012345678
 * ```
 */
export declare class Environment extends pulumi.CustomResource {
    /**
     * Get an existing Environment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EnvironmentState, opts?: pulumi.CustomResourceOptions): Environment;
    /**
     * Returns true if the given object is an instance of Environment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Environment;
    readonly applyChangesDuringMaintenanceWindow: pulumi.Output<boolean | undefined>;
    /**
     * ARN of the Environment.
     */
    readonly arn: pulumi.Output<string>;
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Engine type must be `microfocus` or `bluage`.
     */
    readonly engineType: pulumi.Output<string>;
    /**
     * The specific version of the engine for the Environment.
     */
    readonly engineVersion: pulumi.Output<string>;
    /**
     * The id of the Environment.
     */
    readonly environmentId: pulumi.Output<string>;
    /**
     * Force update the environment even if applications are running.
     */
    readonly forceUpdate: pulumi.Output<boolean | undefined>;
    readonly highAvailabilityConfig: pulumi.Output<outputs.m2.EnvironmentHighAvailabilityConfig | undefined>;
    /**
     * M2 Instance Type.
     *
     * The following arguments are optional:
     */
    readonly instanceType: pulumi.Output<string>;
    /**
     * ARN of the KMS key to use for the Environment.
     */
    readonly kmsKeyId: pulumi.Output<string | undefined>;
    /**
     * ARN of the load balancer created by the Environment.
     */
    readonly loadBalancerArn: pulumi.Output<string>;
    /**
     * Name of the runtime environment. Must be unique within the account.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Configures the maintenance window that you want for the runtime environment. The maintenance window must have the format `ddd:hh24:mi-ddd:hh24:mi` and must be less than 24 hours. If not provided a random value will be used.
     */
    readonly preferredMaintenanceWindow: pulumi.Output<string>;
    /**
     * Allow applications deployed to this environment to be publicly accessible.
     */
    readonly publiclyAccessible: pulumi.Output<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * List of security group ids.
     */
    readonly securityGroupIds: pulumi.Output<string[]>;
    readonly storageConfiguration: pulumi.Output<outputs.m2.EnvironmentStorageConfiguration | undefined>;
    /**
     * List of subnet ids to deploy environment to.
     */
    readonly subnetIds: pulumi.Output<string[]>;
    /**
     * Key-value tags for the place index. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.m2.EnvironmentTimeouts | undefined>;
    /**
     * Create a Environment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EnvironmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Environment resources.
 */
export interface EnvironmentState {
    applyChangesDuringMaintenanceWindow?: pulumi.Input<boolean>;
    /**
     * ARN of the Environment.
     */
    arn?: pulumi.Input<string>;
    description?: pulumi.Input<string>;
    /**
     * Engine type must be `microfocus` or `bluage`.
     */
    engineType?: pulumi.Input<string>;
    /**
     * The specific version of the engine for the Environment.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * The id of the Environment.
     */
    environmentId?: pulumi.Input<string>;
    /**
     * Force update the environment even if applications are running.
     */
    forceUpdate?: pulumi.Input<boolean>;
    highAvailabilityConfig?: pulumi.Input<inputs.m2.EnvironmentHighAvailabilityConfig>;
    /**
     * M2 Instance Type.
     *
     * The following arguments are optional:
     */
    instanceType?: pulumi.Input<string>;
    /**
     * ARN of the KMS key to use for the Environment.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * ARN of the load balancer created by the Environment.
     */
    loadBalancerArn?: pulumi.Input<string>;
    /**
     * Name of the runtime environment. Must be unique within the account.
     */
    name?: pulumi.Input<string>;
    /**
     * Configures the maintenance window that you want for the runtime environment. The maintenance window must have the format `ddd:hh24:mi-ddd:hh24:mi` and must be less than 24 hours. If not provided a random value will be used.
     */
    preferredMaintenanceWindow?: pulumi.Input<string>;
    /**
     * Allow applications deployed to this environment to be publicly accessible.
     */
    publiclyAccessible?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of security group ids.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    storageConfiguration?: pulumi.Input<inputs.m2.EnvironmentStorageConfiguration>;
    /**
     * List of subnet ids to deploy environment to.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value tags for the place index. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.m2.EnvironmentTimeouts>;
}
/**
 * The set of arguments for constructing a Environment resource.
 */
export interface EnvironmentArgs {
    applyChangesDuringMaintenanceWindow?: pulumi.Input<boolean>;
    description?: pulumi.Input<string>;
    /**
     * Engine type must be `microfocus` or `bluage`.
     */
    engineType: pulumi.Input<string>;
    /**
     * The specific version of the engine for the Environment.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * Force update the environment even if applications are running.
     */
    forceUpdate?: pulumi.Input<boolean>;
    highAvailabilityConfig?: pulumi.Input<inputs.m2.EnvironmentHighAvailabilityConfig>;
    /**
     * M2 Instance Type.
     *
     * The following arguments are optional:
     */
    instanceType: pulumi.Input<string>;
    /**
     * ARN of the KMS key to use for the Environment.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Name of the runtime environment. Must be unique within the account.
     */
    name?: pulumi.Input<string>;
    /**
     * Configures the maintenance window that you want for the runtime environment. The maintenance window must have the format `ddd:hh24:mi-ddd:hh24:mi` and must be less than 24 hours. If not provided a random value will be used.
     */
    preferredMaintenanceWindow?: pulumi.Input<string>;
    /**
     * Allow applications deployed to this environment to be publicly accessible.
     */
    publiclyAccessible?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * List of security group ids.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    storageConfiguration?: pulumi.Input<inputs.m2.EnvironmentStorageConfiguration>;
    /**
     * List of subnet ids to deploy environment to.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value tags for the place index. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.m2.EnvironmentTimeouts>;
}
