import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Network Flow Monitor Monitor.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.Vpc("example", {
 *     cidrBlock: "10.0.0.0/16",
 *     tags: {
 *         Name: "example",
 *     },
 * });
 * const exampleMonitor = new aws.networkflowmonitor.Monitor("example", {
 *     monitorName: "example-monitor",
 *     scopeArn: exampleAwsNetworkflowmonitorScope.scopeArn,
 *     localResources: [{
 *         type: "AWS::EC2::VPC",
 *         identifier: example.arn,
 *     }],
 *     remoteResources: [{
 *         type: "AWS::EC2::VPC",
 *         identifier: example.arn,
 *     }],
 *     tags: {
 *         Name: "example",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Network Flow Monitor Monitor using the monitor name. For example:
 *
 * ```sh
 * $ pulumi import aws:networkflowmonitor/monitor:Monitor example example-monitor
 * ```
 */
export declare class Monitor extends pulumi.CustomResource {
    /**
     * Get an existing Monitor resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MonitorState, opts?: pulumi.CustomResourceOptions): Monitor;
    /**
     * Returns true if the given object is an instance of Monitor.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Monitor;
    /**
     * The local resources to monitor. A local resource in a workload is the location of the hosts where the Network Flow Monitor agent is installed.
     */
    readonly localResources: pulumi.Output<outputs.networkflowmonitor.MonitorLocalResource[] | undefined>;
    /**
     * The Amazon Resource Name (ARN) of the monitor.
     */
    readonly monitorArn: pulumi.Output<string>;
    /**
     * The name of the monitor. Cannot be changed after creation.
     */
    readonly monitorName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The remote resources to monitor. A remote resource is the other endpoint specified for the network flow of a workload, with a local resource.
     */
    readonly remoteResources: pulumi.Output<outputs.networkflowmonitor.MonitorRemoteResource[] | undefined>;
    /**
     * The Amazon Resource Name (ARN) of the scope for the monitor. Cannot be changed after creation.
     *
     * The following arguments are optional:
     */
    readonly scopeArn: pulumi.Output<string>;
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
    readonly timeouts: pulumi.Output<outputs.networkflowmonitor.MonitorTimeouts | undefined>;
    /**
     * Create a Monitor resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MonitorArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Monitor resources.
 */
export interface MonitorState {
    /**
     * The local resources to monitor. A local resource in a workload is the location of the hosts where the Network Flow Monitor agent is installed.
     */
    localResources?: pulumi.Input<pulumi.Input<inputs.networkflowmonitor.MonitorLocalResource>[]>;
    /**
     * The Amazon Resource Name (ARN) of the monitor.
     */
    monitorArn?: pulumi.Input<string>;
    /**
     * The name of the monitor. Cannot be changed after creation.
     */
    monitorName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The remote resources to monitor. A remote resource is the other endpoint specified for the network flow of a workload, with a local resource.
     */
    remoteResources?: pulumi.Input<pulumi.Input<inputs.networkflowmonitor.MonitorRemoteResource>[]>;
    /**
     * The Amazon Resource Name (ARN) of the scope for the monitor. Cannot be changed after creation.
     *
     * The following arguments are optional:
     */
    scopeArn?: pulumi.Input<string>;
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
    timeouts?: pulumi.Input<inputs.networkflowmonitor.MonitorTimeouts>;
}
/**
 * The set of arguments for constructing a Monitor resource.
 */
export interface MonitorArgs {
    /**
     * The local resources to monitor. A local resource in a workload is the location of the hosts where the Network Flow Monitor agent is installed.
     */
    localResources?: pulumi.Input<pulumi.Input<inputs.networkflowmonitor.MonitorLocalResource>[]>;
    /**
     * The name of the monitor. Cannot be changed after creation.
     */
    monitorName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The remote resources to monitor. A remote resource is the other endpoint specified for the network flow of a workload, with a local resource.
     */
    remoteResources?: pulumi.Input<pulumi.Input<inputs.networkflowmonitor.MonitorRemoteResource>[]>;
    /**
     * The Amazon Resource Name (ARN) of the scope for the monitor. Cannot be changed after creation.
     *
     * The following arguments are optional:
     */
    scopeArn: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.networkflowmonitor.MonitorTimeouts>;
}
