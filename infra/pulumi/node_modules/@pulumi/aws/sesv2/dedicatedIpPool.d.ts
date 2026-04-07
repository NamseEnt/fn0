import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS SESv2 (Simple Email V2) Dedicated IP Pool.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sesv2.DedicatedIpPool("example", {poolName: "my-pool"});
 * ```
 *
 * ### Managed Pool
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sesv2.DedicatedIpPool("example", {
 *     poolName: "my-managed-pool",
 *     scalingMode: "MANAGED",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SESv2 (Simple Email V2) Dedicated IP Pool using the `pool_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:sesv2/dedicatedIpPool:DedicatedIpPool example my-pool
 * ```
 */
export declare class DedicatedIpPool extends pulumi.CustomResource {
    /**
     * Get an existing DedicatedIpPool resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DedicatedIpPoolState, opts?: pulumi.CustomResourceOptions): DedicatedIpPool;
    /**
     * Returns true if the given object is an instance of DedicatedIpPool.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DedicatedIpPool;
    /**
     * ARN of the Dedicated IP Pool.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Name of the dedicated IP pool.
     *
     * The following arguments are optional:
     */
    readonly poolName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * IP pool scaling mode. Valid values: `STANDARD`, `MANAGED`. If omitted, the AWS API will default to a standard pool.
     */
    readonly scalingMode: pulumi.Output<string>;
    /**
     * A map of tags to assign to the pool. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a DedicatedIpPool resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DedicatedIpPoolArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DedicatedIpPool resources.
 */
export interface DedicatedIpPoolState {
    /**
     * ARN of the Dedicated IP Pool.
     */
    arn?: pulumi.Input<string>;
    /**
     * Name of the dedicated IP pool.
     *
     * The following arguments are optional:
     */
    poolName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * IP pool scaling mode. Valid values: `STANDARD`, `MANAGED`. If omitted, the AWS API will default to a standard pool.
     */
    scalingMode?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the pool. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a DedicatedIpPool resource.
 */
export interface DedicatedIpPoolArgs {
    /**
     * Name of the dedicated IP pool.
     *
     * The following arguments are optional:
     */
    poolName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * IP pool scaling mode. Valid values: `STANDARD`, `MANAGED`. If omitted, the AWS API will default to a standard pool.
     */
    scalingMode?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the pool. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
