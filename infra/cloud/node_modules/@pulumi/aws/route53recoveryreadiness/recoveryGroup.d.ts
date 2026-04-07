import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AWS Route 53 Recovery Readiness Recovery Group.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.route53recoveryreadiness.RecoveryGroup("example", {recoveryGroupName: "my-high-availability-app"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Route53 Recovery Readiness recovery groups using the recovery group name. For example:
 *
 * ```sh
 * $ pulumi import aws:route53recoveryreadiness/recoveryGroup:RecoveryGroup my-high-availability-app my-high-availability-app
 * ```
 */
export declare class RecoveryGroup extends pulumi.CustomResource {
    /**
     * Get an existing RecoveryGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RecoveryGroupState, opts?: pulumi.CustomResourceOptions): RecoveryGroup;
    /**
     * Returns true if the given object is an instance of RecoveryGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RecoveryGroup;
    /**
     * ARN of the recovery group
     */
    readonly arn: pulumi.Output<string>;
    /**
     * List of cell arns to add as nested fault domains within this recovery group
     */
    readonly cells: pulumi.Output<string[] | undefined>;
    /**
     * A unique name describing the recovery group.
     *
     * The following arguments are optional:
     */
    readonly recoveryGroupName: pulumi.Output<string>;
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
     * Create a RecoveryGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RecoveryGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RecoveryGroup resources.
 */
export interface RecoveryGroupState {
    /**
     * ARN of the recovery group
     */
    arn?: pulumi.Input<string>;
    /**
     * List of cell arns to add as nested fault domains within this recovery group
     */
    cells?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A unique name describing the recovery group.
     *
     * The following arguments are optional:
     */
    recoveryGroupName?: pulumi.Input<string>;
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
 * The set of arguments for constructing a RecoveryGroup resource.
 */
export interface RecoveryGroupArgs {
    /**
     * List of cell arns to add as nested fault domains within this recovery group
     */
    cells?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A unique name describing the recovery group.
     *
     * The following arguments are optional:
     */
    recoveryGroupName: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
