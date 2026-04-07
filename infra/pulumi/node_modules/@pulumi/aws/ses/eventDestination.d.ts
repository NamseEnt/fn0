import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an SES event destination
 *
 * ## Example Usage
 *
 * ### CloudWatch Destination
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const cloudwatch = new aws.ses.EventDestination("cloudwatch", {
 *     name: "event-destination-cloudwatch",
 *     configurationSetName: example.name,
 *     enabled: true,
 *     matchingTypes: [
 *         "bounce",
 *         "send",
 *     ],
 *     cloudwatchDestinations: [{
 *         defaultValue: "default",
 *         dimensionName: "dimension",
 *         valueSource: "emailHeader",
 *     }],
 * });
 * ```
 *
 * ### Kinesis Destination
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const kinesis = new aws.ses.EventDestination("kinesis", {
 *     name: "event-destination-kinesis",
 *     configurationSetName: exampleAwsSesConfigurationSet.name,
 *     enabled: true,
 *     matchingTypes: [
 *         "bounce",
 *         "send",
 *     ],
 *     kinesisDestination: {
 *         streamArn: exampleAwsKinesisFirehoseDeliveryStream.arn,
 *         roleArn: example.arn,
 *     },
 * });
 * ```
 *
 * ### SNS Destination
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const sns = new aws.ses.EventDestination("sns", {
 *     name: "event-destination-sns",
 *     configurationSetName: exampleAwsSesConfigurationSet.name,
 *     enabled: true,
 *     matchingTypes: [
 *         "bounce",
 *         "send",
 *     ],
 *     snsDestination: {
 *         topicArn: example.arn,
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SES event destinations using `configuration_set_name` together with the event destination's `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:ses/eventDestination:EventDestination sns some-configuration-set-test/event-destination-sns
 * ```
 */
export declare class EventDestination extends pulumi.CustomResource {
    /**
     * Get an existing EventDestination resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EventDestinationState, opts?: pulumi.CustomResourceOptions): EventDestination;
    /**
     * Returns true if the given object is an instance of EventDestination.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EventDestination;
    /**
     * The SES event destination ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * CloudWatch destination for the events
     */
    readonly cloudwatchDestinations: pulumi.Output<outputs.ses.EventDestinationCloudwatchDestination[] | undefined>;
    /**
     * The name of the configuration set
     */
    readonly configurationSetName: pulumi.Output<string>;
    /**
     * If true, the event destination will be enabled
     */
    readonly enabled: pulumi.Output<boolean | undefined>;
    /**
     * Send the events to a kinesis firehose destination
     */
    readonly kinesisDestination: pulumi.Output<outputs.ses.EventDestinationKinesisDestination | undefined>;
    /**
     * A list of matching types. May be any of `"send"`, `"reject"`, `"bounce"`, `"complaint"`, `"delivery"`, `"open"`, `"click"`, or `"renderingFailure"`.
     */
    readonly matchingTypes: pulumi.Output<string[]>;
    /**
     * The name of the event destination
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Send the events to an SNS Topic destination
     *
     * > **NOTE:** You can specify `"cloudwatchDestination"` or `"kinesisDestination"` but not both
     */
    readonly snsDestination: pulumi.Output<outputs.ses.EventDestinationSnsDestination | undefined>;
    /**
     * Create a EventDestination resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EventDestinationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EventDestination resources.
 */
export interface EventDestinationState {
    /**
     * The SES event destination ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * CloudWatch destination for the events
     */
    cloudwatchDestinations?: pulumi.Input<pulumi.Input<inputs.ses.EventDestinationCloudwatchDestination>[]>;
    /**
     * The name of the configuration set
     */
    configurationSetName?: pulumi.Input<string>;
    /**
     * If true, the event destination will be enabled
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Send the events to a kinesis firehose destination
     */
    kinesisDestination?: pulumi.Input<inputs.ses.EventDestinationKinesisDestination>;
    /**
     * A list of matching types. May be any of `"send"`, `"reject"`, `"bounce"`, `"complaint"`, `"delivery"`, `"open"`, `"click"`, or `"renderingFailure"`.
     */
    matchingTypes?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name of the event destination
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Send the events to an SNS Topic destination
     *
     * > **NOTE:** You can specify `"cloudwatchDestination"` or `"kinesisDestination"` but not both
     */
    snsDestination?: pulumi.Input<inputs.ses.EventDestinationSnsDestination>;
}
/**
 * The set of arguments for constructing a EventDestination resource.
 */
export interface EventDestinationArgs {
    /**
     * CloudWatch destination for the events
     */
    cloudwatchDestinations?: pulumi.Input<pulumi.Input<inputs.ses.EventDestinationCloudwatchDestination>[]>;
    /**
     * The name of the configuration set
     */
    configurationSetName: pulumi.Input<string>;
    /**
     * If true, the event destination will be enabled
     */
    enabled?: pulumi.Input<boolean>;
    /**
     * Send the events to a kinesis firehose destination
     */
    kinesisDestination?: pulumi.Input<inputs.ses.EventDestinationKinesisDestination>;
    /**
     * A list of matching types. May be any of `"send"`, `"reject"`, `"bounce"`, `"complaint"`, `"delivery"`, `"open"`, `"click"`, or `"renderingFailure"`.
     */
    matchingTypes: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name of the event destination
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Send the events to an SNS Topic destination
     *
     * > **NOTE:** You can specify `"cloudwatchDestination"` or `"kinesisDestination"` but not both
     */
    snsDestination?: pulumi.Input<inputs.ses.EventDestinationSnsDestination>;
}
