import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an [AWS AppSync Channel Namespace](https://docs.aws.amazon.com/appsync/latest/eventapi/event-api-concepts.html#namespace).
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appsync.ChannelNamespace("example", {
 *     name: "example-channel-namespace",
 *     apiId: exampleAwsAppsyncApi.apiId,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import AppSync Channel Namespace using the `api_id` and `name` separated by a comma (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:appsync/channelNamespace:ChannelNamespace example example-api-id,example-channel-namespace
 * ```
 */
export declare class ChannelNamespace extends pulumi.CustomResource {
    /**
     * Get an existing ChannelNamespace resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ChannelNamespaceState, opts?: pulumi.CustomResourceOptions): ChannelNamespace;
    /**
     * Returns true if the given object is an instance of ChannelNamespace.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ChannelNamespace;
    /**
     * Event API ID.
     */
    readonly apiId: pulumi.Output<string>;
    /**
     * ARN of the channel namespace.
     */
    readonly channelNamespaceArn: pulumi.Output<string>;
    /**
     * Event handler functions that run custom business logic to process published events and subscribe requests.
     */
    readonly codeHandlers: pulumi.Output<string | undefined>;
    /**
     * Configuration for the `onPublish` and `onSubscribe` handlers. See Handler Configs below.
     */
    readonly handlerConfigs: pulumi.Output<outputs.appsync.ChannelNamespaceHandlerConfigs | undefined>;
    /**
     * Name of the channel namespace.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Authorization modes to use for publishing messages on the channel namespace. This configuration overrides the default API authorization configuration. See Auth Modes below.
     */
    readonly publishAuthModes: pulumi.Output<outputs.appsync.ChannelNamespacePublishAuthMode[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Authorization modes to use for subscribing to messages on the channel namespace. This configuration overrides the default API authorization configuration. See Auth Modes below.
     */
    readonly subscribeAuthModes: pulumi.Output<outputs.appsync.ChannelNamespaceSubscribeAuthMode[] | undefined>;
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
     * Create a ChannelNamespace resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ChannelNamespaceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ChannelNamespace resources.
 */
export interface ChannelNamespaceState {
    /**
     * Event API ID.
     */
    apiId?: pulumi.Input<string>;
    /**
     * ARN of the channel namespace.
     */
    channelNamespaceArn?: pulumi.Input<string>;
    /**
     * Event handler functions that run custom business logic to process published events and subscribe requests.
     */
    codeHandlers?: pulumi.Input<string>;
    /**
     * Configuration for the `onPublish` and `onSubscribe` handlers. See Handler Configs below.
     */
    handlerConfigs?: pulumi.Input<inputs.appsync.ChannelNamespaceHandlerConfigs>;
    /**
     * Name of the channel namespace.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Authorization modes to use for publishing messages on the channel namespace. This configuration overrides the default API authorization configuration. See Auth Modes below.
     */
    publishAuthModes?: pulumi.Input<pulumi.Input<inputs.appsync.ChannelNamespacePublishAuthMode>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Authorization modes to use for subscribing to messages on the channel namespace. This configuration overrides the default API authorization configuration. See Auth Modes below.
     */
    subscribeAuthModes?: pulumi.Input<pulumi.Input<inputs.appsync.ChannelNamespaceSubscribeAuthMode>[]>;
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
 * The set of arguments for constructing a ChannelNamespace resource.
 */
export interface ChannelNamespaceArgs {
    /**
     * Event API ID.
     */
    apiId: pulumi.Input<string>;
    /**
     * Event handler functions that run custom business logic to process published events and subscribe requests.
     */
    codeHandlers?: pulumi.Input<string>;
    /**
     * Configuration for the `onPublish` and `onSubscribe` handlers. See Handler Configs below.
     */
    handlerConfigs?: pulumi.Input<inputs.appsync.ChannelNamespaceHandlerConfigs>;
    /**
     * Name of the channel namespace.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Authorization modes to use for publishing messages on the channel namespace. This configuration overrides the default API authorization configuration. See Auth Modes below.
     */
    publishAuthModes?: pulumi.Input<pulumi.Input<inputs.appsync.ChannelNamespacePublishAuthMode>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Authorization modes to use for subscribing to messages on the channel namespace. This configuration overrides the default API authorization configuration. See Auth Modes below.
     */
    subscribeAuthModes?: pulumi.Input<pulumi.Input<inputs.appsync.ChannelNamespaceSubscribeAuthMode>[]>;
    /**
     * Map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
