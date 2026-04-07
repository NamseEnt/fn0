import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AppConfig Environment resource for an `aws.appconfig.Application` resource. One or more environments can be defined for an application.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const exampleApplication = new aws.appconfig.Application("example", {
 *     name: "example-application-tf",
 *     description: "Example AppConfig Application",
 *     tags: {
 *         Type: "AppConfig Application",
 *     },
 * });
 * const example = new aws.appconfig.Environment("example", {
 *     name: "example-environment-tf",
 *     description: "Example AppConfig Environment",
 *     applicationId: exampleApplication.id,
 *     monitors: [{
 *         alarmArn: exampleAwsCloudwatchMetricAlarm.arn,
 *         alarmRoleArn: exampleAwsIamRole.arn,
 *     }],
 *     tags: {
 *         Type: "AppConfig Environment",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AppConfig Environments using the environment ID and application ID separated by a colon (`:`). For example:
 *
 * ```sh
 * $ pulumi import aws:appconfig/environment:Environment example 71abcde:11xxxxx
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
    /**
     * AppConfig application ID. Must be between 4 and 7 characters in length.
     */
    readonly applicationId: pulumi.Output<string>;
    /**
     * ARN of the AppConfig Environment.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Description of the environment. Can be at most 1024 characters.
     */
    readonly description: pulumi.Output<string>;
    /**
     * AppConfig environment ID.
     */
    readonly environmentId: pulumi.Output<string>;
    /**
     * Set of Amazon CloudWatch alarms to monitor during the deployment process. Maximum of 5. See Monitor below for more details.
     */
    readonly monitors: pulumi.Output<outputs.appconfig.EnvironmentMonitor[] | undefined>;
    /**
     * Name for the environment. Must be between 1 and 64 characters in length.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * State of the environment. Possible values are `READY_FOR_DEPLOYMENT`, `DEPLOYING`, `ROLLING_BACK`
     * or `ROLLED_BACK`.
     */
    readonly state: pulumi.Output<string>;
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
    /**
     * AppConfig application ID. Must be between 4 and 7 characters in length.
     */
    applicationId?: pulumi.Input<string>;
    /**
     * ARN of the AppConfig Environment.
     */
    arn?: pulumi.Input<string>;
    /**
     * Description of the environment. Can be at most 1024 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * AppConfig environment ID.
     */
    environmentId?: pulumi.Input<string>;
    /**
     * Set of Amazon CloudWatch alarms to monitor during the deployment process. Maximum of 5. See Monitor below for more details.
     */
    monitors?: pulumi.Input<pulumi.Input<inputs.appconfig.EnvironmentMonitor>[]>;
    /**
     * Name for the environment. Must be between 1 and 64 characters in length.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * State of the environment. Possible values are `READY_FOR_DEPLOYMENT`, `DEPLOYING`, `ROLLING_BACK`
     * or `ROLLED_BACK`.
     */
    state?: pulumi.Input<string>;
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
 * The set of arguments for constructing a Environment resource.
 */
export interface EnvironmentArgs {
    /**
     * AppConfig application ID. Must be between 4 and 7 characters in length.
     */
    applicationId: pulumi.Input<string>;
    /**
     * Description of the environment. Can be at most 1024 characters.
     */
    description?: pulumi.Input<string>;
    /**
     * Set of Amazon CloudWatch alarms to monitor during the deployment process. Maximum of 5. See Monitor below for more details.
     */
    monitors?: pulumi.Input<pulumi.Input<inputs.appconfig.EnvironmentMonitor>[]>;
    /**
     * Name for the environment. Must be between 1 and 64 characters in length.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
