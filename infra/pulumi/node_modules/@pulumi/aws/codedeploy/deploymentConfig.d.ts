import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a CodeDeploy deployment config for an application
 *
 * ## Example Usage
 *
 * ### Server Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const foo = new aws.codedeploy.DeploymentConfig("foo", {
 *     deploymentConfigName: "test-deployment-config",
 *     minimumHealthyHosts: {
 *         type: "HOST_COUNT",
 *         value: 2,
 *     },
 * });
 * const fooDeploymentGroup = new aws.codedeploy.DeploymentGroup("foo", {
 *     appName: fooApp.name,
 *     deploymentGroupName: "bar",
 *     serviceRoleArn: fooRole.arn,
 *     deploymentConfigName: foo.id,
 *     ec2TagFilters: [{
 *         key: "filterkey",
 *         type: "KEY_AND_VALUE",
 *         value: "filtervalue",
 *     }],
 *     triggerConfigurations: [{
 *         triggerEvents: ["DeploymentFailure"],
 *         triggerName: "foo-trigger",
 *         triggerTargetArn: "foo-topic-arn",
 *     }],
 *     autoRollbackConfiguration: {
 *         enabled: true,
 *         events: ["DEPLOYMENT_FAILURE"],
 *     },
 *     alarmConfiguration: {
 *         alarms: ["my-alarm-name"],
 *         enabled: true,
 *     },
 * });
 * ```
 *
 * ### Lambda Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const foo = new aws.codedeploy.DeploymentConfig("foo", {
 *     deploymentConfigName: "test-deployment-config",
 *     computePlatform: "Lambda",
 *     trafficRoutingConfig: {
 *         type: "TimeBasedLinear",
 *         timeBasedLinear: {
 *             interval: 10,
 *             percentage: 10,
 *         },
 *     },
 * });
 * const fooDeploymentGroup = new aws.codedeploy.DeploymentGroup("foo", {
 *     appName: fooApp.name,
 *     deploymentGroupName: "bar",
 *     serviceRoleArn: fooRole.arn,
 *     deploymentConfigName: foo.id,
 *     autoRollbackConfiguration: {
 *         enabled: true,
 *         events: ["DEPLOYMENT_STOP_ON_ALARM"],
 *     },
 *     alarmConfiguration: {
 *         alarms: ["my-alarm-name"],
 *         enabled: true,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import CodeDeploy Deployment Configurations using the `deployment_config_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:codedeploy/deploymentConfig:DeploymentConfig example my-deployment-config
 * ```
 */
export declare class DeploymentConfig extends pulumi.CustomResource {
    /**
     * Get an existing DeploymentConfig resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DeploymentConfigState, opts?: pulumi.CustomResourceOptions): DeploymentConfig;
    /**
     * Returns true if the given object is an instance of DeploymentConfig.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DeploymentConfig;
    /**
     * The ARN of the deployment config.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The compute platform can be `Server`, `Lambda`, or `ECS`. Default is `Server`.
     */
    readonly computePlatform: pulumi.Output<string | undefined>;
    /**
     * The AWS Assigned deployment config id
     */
    readonly deploymentConfigId: pulumi.Output<string>;
    /**
     * The name of the deployment config.
     */
    readonly deploymentConfigName: pulumi.Output<string>;
    /**
     * A minimumHealthyHosts block. Required for `Server` compute platform. Minimum Healthy Hosts are documented below.
     */
    readonly minimumHealthyHosts: pulumi.Output<outputs.codedeploy.DeploymentConfigMinimumHealthyHosts | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A trafficRoutingConfig block. Traffic Routing Config is documented below.
     */
    readonly trafficRoutingConfig: pulumi.Output<outputs.codedeploy.DeploymentConfigTrafficRoutingConfig | undefined>;
    /**
     * A zonalConfig block. Zonal Config is documented below.
     */
    readonly zonalConfig: pulumi.Output<outputs.codedeploy.DeploymentConfigZonalConfig | undefined>;
    /**
     * Create a DeploymentConfig resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: DeploymentConfigArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DeploymentConfig resources.
 */
export interface DeploymentConfigState {
    /**
     * The ARN of the deployment config.
     */
    arn?: pulumi.Input<string>;
    /**
     * The compute platform can be `Server`, `Lambda`, or `ECS`. Default is `Server`.
     */
    computePlatform?: pulumi.Input<string>;
    /**
     * The AWS Assigned deployment config id
     */
    deploymentConfigId?: pulumi.Input<string>;
    /**
     * The name of the deployment config.
     */
    deploymentConfigName?: pulumi.Input<string>;
    /**
     * A minimumHealthyHosts block. Required for `Server` compute platform. Minimum Healthy Hosts are documented below.
     */
    minimumHealthyHosts?: pulumi.Input<inputs.codedeploy.DeploymentConfigMinimumHealthyHosts>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A trafficRoutingConfig block. Traffic Routing Config is documented below.
     */
    trafficRoutingConfig?: pulumi.Input<inputs.codedeploy.DeploymentConfigTrafficRoutingConfig>;
    /**
     * A zonalConfig block. Zonal Config is documented below.
     */
    zonalConfig?: pulumi.Input<inputs.codedeploy.DeploymentConfigZonalConfig>;
}
/**
 * The set of arguments for constructing a DeploymentConfig resource.
 */
export interface DeploymentConfigArgs {
    /**
     * The compute platform can be `Server`, `Lambda`, or `ECS`. Default is `Server`.
     */
    computePlatform?: pulumi.Input<string>;
    /**
     * The name of the deployment config.
     */
    deploymentConfigName?: pulumi.Input<string>;
    /**
     * A minimumHealthyHosts block. Required for `Server` compute platform. Minimum Healthy Hosts are documented below.
     */
    minimumHealthyHosts?: pulumi.Input<inputs.codedeploy.DeploymentConfigMinimumHealthyHosts>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A trafficRoutingConfig block. Traffic Routing Config is documented below.
     */
    trafficRoutingConfig?: pulumi.Input<inputs.codedeploy.DeploymentConfigTrafficRoutingConfig>;
    /**
     * A zonalConfig block. Zonal Config is documented below.
     */
    zonalConfig?: pulumi.Input<inputs.codedeploy.DeploymentConfigZonalConfig>;
}
