import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Data Exchange Event Action.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.dataexchange.EventAction("example", {
 *     event: {
 *         revisionPublished: {
 *             dataSetId: exampleAwsDataexchangeDataSet.id,
 *         },
 *     },
 *     action: {
 *         exportRevisionToS3: {
 *             revisionDestination: {
 *                 bucket: exampleAwsS3Bucket.bucket,
 *                 keyPattern: "${Revision.CreatedAt}/${Asset.Name}",
 *             },
 *             encryption: {
 *                 type: "aws:kms",
 *                 kmsKeyArn: exampleAwsKmsKey.arn,
 *             },
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Data Exchange Event Action using the id. For example:
 *
 * ```sh
 * $ pulumi import aws:dataexchange/eventAction:EventAction example example-event-action-id
 * ```
 */
export declare class EventAction extends pulumi.CustomResource {
    /**
     * Get an existing EventAction resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EventActionState, opts?: pulumi.CustomResourceOptions): EventAction;
    /**
     * Returns true if the given object is an instance of EventAction.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EventAction;
    /**
     * Describes the action to take.
     * Described in `action` Configuration Block below.
     */
    readonly action: pulumi.Output<outputs.dataexchange.EventActionAction | undefined>;
    /**
     * Amazon Resource Name (ARN) of the event action.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Date and time when the resource was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Describes the event that triggers the `action`.
     * Described in `event` Configuration Block below.
     */
    readonly event: pulumi.Output<outputs.dataexchange.EventActionEvent | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Data and time when the resource was last updated.
     */
    readonly updatedAt: pulumi.Output<string>;
    /**
     * Create a EventAction resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: EventActionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EventAction resources.
 */
export interface EventActionState {
    /**
     * Describes the action to take.
     * Described in `action` Configuration Block below.
     */
    action?: pulumi.Input<inputs.dataexchange.EventActionAction>;
    /**
     * Amazon Resource Name (ARN) of the event action.
     */
    arn?: pulumi.Input<string>;
    /**
     * Date and time when the resource was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Describes the event that triggers the `action`.
     * Described in `event` Configuration Block below.
     */
    event?: pulumi.Input<inputs.dataexchange.EventActionEvent>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Data and time when the resource was last updated.
     */
    updatedAt?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EventAction resource.
 */
export interface EventActionArgs {
    /**
     * Describes the action to take.
     * Described in `action` Configuration Block below.
     */
    action?: pulumi.Input<inputs.dataexchange.EventActionAction>;
    /**
     * Describes the event that triggers the `action`.
     * Described in `event` Configuration Block below.
     */
    event?: pulumi.Input<inputs.dataexchange.EventActionEvent>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
