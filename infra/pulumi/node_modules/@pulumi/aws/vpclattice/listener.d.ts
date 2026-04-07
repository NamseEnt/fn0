import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS VPC Lattice Listener.
 *
 * ## Example Usage
 *
 * ### Fixed response action
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpclattice.Service("example", {name: "example"});
 * const exampleListener = new aws.vpclattice.Listener("example", {
 *     name: "example",
 *     protocol: "HTTPS",
 *     serviceIdentifier: example.id,
 *     defaultAction: {
 *         fixedResponse: {
 *             statusCode: 404,
 *         },
 *     },
 * });
 * ```
 *
 * ### Forward action
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpclattice.Service("example", {name: "example"});
 * const exampleTargetGroup = new aws.vpclattice.TargetGroup("example", {
 *     name: "example-target-group-1",
 *     type: "INSTANCE",
 *     config: {
 *         port: 80,
 *         protocol: "HTTP",
 *         vpcIdentifier: exampleAwsVpc.id,
 *     },
 * });
 * const exampleListener = new aws.vpclattice.Listener("example", {
 *     name: "example",
 *     protocol: "HTTP",
 *     serviceIdentifier: example.id,
 *     defaultAction: {
 *         forwards: [{
 *             targetGroups: [{
 *                 targetGroupIdentifier: exampleTargetGroup.id,
 *             }],
 *         }],
 *     },
 * });
 * ```
 *
 * ### Forward action with weighted target groups
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpclattice.Service("example", {name: "example"});
 * const example1 = new aws.vpclattice.TargetGroup("example1", {
 *     name: "example-target-group-1",
 *     type: "INSTANCE",
 *     config: {
 *         port: 80,
 *         protocol: "HTTP",
 *         vpcIdentifier: exampleAwsVpc.id,
 *     },
 * });
 * const example2 = new aws.vpclattice.TargetGroup("example2", {
 *     name: "example-target-group-2",
 *     type: "INSTANCE",
 *     config: {
 *         port: 8080,
 *         protocol: "HTTP",
 *         vpcIdentifier: exampleAwsVpc.id,
 *     },
 * });
 * const exampleListener = new aws.vpclattice.Listener("example", {
 *     name: "example",
 *     protocol: "HTTP",
 *     serviceIdentifier: example.id,
 *     defaultAction: {
 *         forwards: [{
 *             targetGroups: [
 *                 {
 *                     targetGroupIdentifier: example1.id,
 *                     weight: 80,
 *                 },
 *                 {
 *                     targetGroupIdentifier: example2.id,
 *                     weight: 20,
 *                 },
 *             ],
 *         }],
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC Lattice Listener using the `listener_id` of the listener and the `id` of the VPC Lattice service combined with a `/` character. For example:
 *
 * ```sh
 * $ pulumi import aws:vpclattice/listener:Listener example svc-1a2b3c4d/listener-987654321
 * ```
 */
export declare class Listener extends pulumi.CustomResource {
    /**
     * Get an existing Listener resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ListenerState, opts?: pulumi.CustomResourceOptions): Listener;
    /**
     * Returns true if the given object is an instance of Listener.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Listener;
    /**
     * ARN of the listener.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Date and time that the listener was created, specified in ISO-8601 format.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Default action block for the default listener rule. Default action blocks are defined below.
     */
    readonly defaultAction: pulumi.Output<outputs.vpclattice.ListenerDefaultAction>;
    readonly lastUpdatedAt: pulumi.Output<string>;
    /**
     * Standalone ID of the listener, e.g. `listener-0a1b2c3d4e5f6g`.
     */
    readonly listenerId: pulumi.Output<string>;
    /**
     * Name of the listener. A listener name must be unique within a service. Valid characters are a-z, 0-9, and hyphens (-). You can't use a hyphen as the first or last character, or immediately after another hyphen.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Listener port. You can specify a value from 1 to 65535. If `port` is not specified and `protocol` is HTTP, the value will default to 80. If `port` is not specified and `protocol` is HTTPS, the value will default to 443.
     */
    readonly port: pulumi.Output<number>;
    /**
     * Protocol for the listener. Supported values are `HTTP`, `HTTPS` or `TLS_PASSTHROUGH`
     */
    readonly protocol: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the VPC Lattice service. You must include either the `serviceArn` or `serviceIdentifier` arguments.
     */
    readonly serviceArn: pulumi.Output<string>;
    /**
     * ID of the VPC Lattice service. You must include either the `serviceArn` or `serviceIdentifier` arguments.
     * > **NOTE:** You must specify one of the following arguments: `serviceArn` or `serviceIdentifier`.
     */
    readonly serviceIdentifier: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a Listener resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ListenerArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Listener resources.
 */
export interface ListenerState {
    /**
     * ARN of the listener.
     */
    arn?: pulumi.Input<string>;
    /**
     * Date and time that the listener was created, specified in ISO-8601 format.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Default action block for the default listener rule. Default action blocks are defined below.
     */
    defaultAction?: pulumi.Input<inputs.vpclattice.ListenerDefaultAction>;
    lastUpdatedAt?: pulumi.Input<string>;
    /**
     * Standalone ID of the listener, e.g. `listener-0a1b2c3d4e5f6g`.
     */
    listenerId?: pulumi.Input<string>;
    /**
     * Name of the listener. A listener name must be unique within a service. Valid characters are a-z, 0-9, and hyphens (-). You can't use a hyphen as the first or last character, or immediately after another hyphen.
     */
    name?: pulumi.Input<string>;
    /**
     * Listener port. You can specify a value from 1 to 65535. If `port` is not specified and `protocol` is HTTP, the value will default to 80. If `port` is not specified and `protocol` is HTTPS, the value will default to 443.
     */
    port?: pulumi.Input<number>;
    /**
     * Protocol for the listener. Supported values are `HTTP`, `HTTPS` or `TLS_PASSTHROUGH`
     */
    protocol?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the VPC Lattice service. You must include either the `serviceArn` or `serviceIdentifier` arguments.
     */
    serviceArn?: pulumi.Input<string>;
    /**
     * ID of the VPC Lattice service. You must include either the `serviceArn` or `serviceIdentifier` arguments.
     * > **NOTE:** You must specify one of the following arguments: `serviceArn` or `serviceIdentifier`.
     */
    serviceIdentifier?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a Listener resource.
 */
export interface ListenerArgs {
    /**
     * Default action block for the default listener rule. Default action blocks are defined below.
     */
    defaultAction: pulumi.Input<inputs.vpclattice.ListenerDefaultAction>;
    /**
     * Name of the listener. A listener name must be unique within a service. Valid characters are a-z, 0-9, and hyphens (-). You can't use a hyphen as the first or last character, or immediately after another hyphen.
     */
    name?: pulumi.Input<string>;
    /**
     * Listener port. You can specify a value from 1 to 65535. If `port` is not specified and `protocol` is HTTP, the value will default to 80. If `port` is not specified and `protocol` is HTTPS, the value will default to 443.
     */
    port?: pulumi.Input<number>;
    /**
     * Protocol for the listener. Supported values are `HTTP`, `HTTPS` or `TLS_PASSTHROUGH`
     */
    protocol: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the VPC Lattice service. You must include either the `serviceArn` or `serviceIdentifier` arguments.
     */
    serviceArn?: pulumi.Input<string>;
    /**
     * ID of the VPC Lattice service. You must include either the `serviceArn` or `serviceIdentifier` arguments.
     * > **NOTE:** You must specify one of the following arguments: `serviceArn` or `serviceIdentifier`.
     */
    serviceIdentifier?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
