import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an AWS Route 53 Recovery Readiness Resource Set.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.route53recoveryreadiness.ResourceSet("example", {
 *     resourceSetName: my_cw_alarm_set,
 *     resourceSetType: "AWS::CloudWatch::Alarm",
 *     resources: [{
 *         resourceArn: exampleAwsCloudwatchMetricAlarm.arn,
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Route53 Recovery Readiness resource set name using the resource set name. For example:
 *
 * ```sh
 * $ pulumi import aws:route53recoveryreadiness/resourceSet:ResourceSet my-cw-alarm-set example
 * ```
 */
export declare class ResourceSet extends pulumi.CustomResource {
    /**
     * Get an existing ResourceSet resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResourceSetState, opts?: pulumi.CustomResourceOptions): ResourceSet;
    /**
     * Returns true if the given object is an instance of ResourceSet.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResourceSet;
    /**
     * ARN of the resource set
     * * `resources.#.component_id` - Unique identified for DNS Target Resources, use for readiness checks.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Unique name describing the resource set.
     */
    readonly resourceSetName: pulumi.Output<string>;
    /**
     * Type of the resources in the resource set.
     */
    readonly resourceSetType: pulumi.Output<string>;
    /**
     * List of resources to add to this resource set. See below.
     *
     * The following arguments are optional:
     */
    readonly resources: pulumi.Output<outputs.route53recoveryreadiness.ResourceSetResource[]>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
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
     * Create a ResourceSet resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ResourceSetArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResourceSet resources.
 */
export interface ResourceSetState {
    /**
     * ARN of the resource set
     * * `resources.#.component_id` - Unique identified for DNS Target Resources, use for readiness checks.
     */
    arn?: pulumi.Input<string>;
    /**
     * Unique name describing the resource set.
     */
    resourceSetName?: pulumi.Input<string>;
    /**
     * Type of the resources in the resource set.
     */
    resourceSetType?: pulumi.Input<string>;
    /**
     * List of resources to add to this resource set. See below.
     *
     * The following arguments are optional:
     */
    resources?: pulumi.Input<pulumi.Input<inputs.route53recoveryreadiness.ResourceSetResource>[]>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
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
 * The set of arguments for constructing a ResourceSet resource.
 */
export interface ResourceSetArgs {
    /**
     * Unique name describing the resource set.
     */
    resourceSetName: pulumi.Input<string>;
    /**
     * Type of the resources in the resource set.
     */
    resourceSetType: pulumi.Input<string>;
    /**
     * List of resources to add to this resource set. See below.
     *
     * The following arguments are optional:
     */
    resources: pulumi.Input<pulumi.Input<inputs.route53recoveryreadiness.ResourceSetResource>[]>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
