import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Amazon AppIntegrations Event Integration resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appconfig.EventIntegration("example", {
 *     name: "example-name",
 *     description: "Example Description",
 *     eventbridgeBus: "default",
 *     eventFilter: {
 *         source: "aws.partner/examplepartner.com",
 *     },
 *     tags: {
 *         Name: "Example Event Integration",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Amazon AppIntegrations Event Integrations using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:appconfig/eventIntegration:EventIntegration example example-name
 * ```
 */
export declare class EventIntegration extends pulumi.CustomResource {
    /**
     * Get an existing EventIntegration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EventIntegrationState, opts?: pulumi.CustomResourceOptions): EventIntegration;
    /**
     * Returns true if the given object is an instance of EventIntegration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EventIntegration;
    /**
     * ARN of the Event Integration.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Description of the Event Integration.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Block that defines the configuration information for the event filter. The Event Filter block is documented below.
     */
    readonly eventFilter: pulumi.Output<outputs.appconfig.EventIntegrationEventFilter>;
    /**
     * EventBridge bus.
     */
    readonly eventbridgeBus: pulumi.Output<string>;
    /**
     * Name of the Event Integration.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Tags to apply to the Event Integration. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a EventIntegration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EventIntegrationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EventIntegration resources.
 */
export interface EventIntegrationState {
    /**
     * ARN of the Event Integration.
     */
    arn?: pulumi.Input<string>;
    /**
     * Description of the Event Integration.
     */
    description?: pulumi.Input<string>;
    /**
     * Block that defines the configuration information for the event filter. The Event Filter block is documented below.
     */
    eventFilter?: pulumi.Input<inputs.appconfig.EventIntegrationEventFilter>;
    /**
     * EventBridge bus.
     */
    eventbridgeBus?: pulumi.Input<string>;
    /**
     * Name of the Event Integration.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags to apply to the Event Integration. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a EventIntegration resource.
 */
export interface EventIntegrationArgs {
    /**
     * Description of the Event Integration.
     */
    description?: pulumi.Input<string>;
    /**
     * Block that defines the configuration information for the event filter. The Event Filter block is documented below.
     */
    eventFilter: pulumi.Input<inputs.appconfig.EventIntegrationEventFilter>;
    /**
     * EventBridge bus.
     */
    eventbridgeBus: pulumi.Input<string>;
    /**
     * Name of the Event Integration.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags to apply to the Event Integration. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
