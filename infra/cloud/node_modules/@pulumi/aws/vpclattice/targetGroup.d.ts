import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS VPC Lattice Target Group.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpclattice.TargetGroup("example", {
 *     name: "example",
 *     type: "INSTANCE",
 *     config: {
 *         vpcIdentifier: exampleAwsVpc.id,
 *         port: 443,
 *         protocol: "HTTPS",
 *     },
 * });
 * ```
 *
 * ### Basic usage with Health check
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpclattice.TargetGroup("example", {
 *     name: "example",
 *     type: "IP",
 *     config: {
 *         vpcIdentifier: exampleAwsVpc.id,
 *         ipAddressType: "IPV4",
 *         port: 443,
 *         protocol: "HTTPS",
 *         protocolVersion: "HTTP1",
 *         healthCheck: {
 *             enabled: true,
 *             healthCheckIntervalSeconds: 20,
 *             healthCheckTimeoutSeconds: 10,
 *             healthyThresholdCount: 7,
 *             unhealthyThresholdCount: 3,
 *             matcher: {
 *                 value: "200-299",
 *             },
 *             path: "/instance",
 *             port: 80,
 *             protocol: "HTTP",
 *             protocolVersion: "HTTP1",
 *         },
 *     },
 * });
 * ```
 *
 * ### ALB
 *
 * If the type is ALB, `healthCheck` block is not supported.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpclattice.TargetGroup("example", {
 *     name: "example",
 *     type: "ALB",
 *     config: {
 *         vpcIdentifier: exampleAwsVpc.id,
 *         port: 443,
 *         protocol: "HTTPS",
 *         protocolVersion: "HTTP1",
 *     },
 * });
 * ```
 *
 * ### Lambda
 *
 * If the type is Lambda, `config` block is not supported.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpclattice.TargetGroup("example", {
 *     name: "example",
 *     type: "LAMBDA",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC Lattice Target Group using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:vpclattice/targetGroup:TargetGroup example tg-0c11d4dc16ed96bdb
 * ```
 */
export declare class TargetGroup extends pulumi.CustomResource {
    /**
     * Get an existing TargetGroup resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TargetGroupState, opts?: pulumi.CustomResourceOptions): TargetGroup;
    /**
     * Returns true if the given object is an instance of TargetGroup.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TargetGroup;
    /**
     * ARN of the target group.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The target group configuration.
     */
    readonly config: pulumi.Output<outputs.vpclattice.TargetGroupConfig | undefined>;
    /**
     * The name of the target group. The name must be unique within the account. The valid characters are a-z, 0-9, and hyphens (-). You can't use a hyphen as the first or last character, or immediately after another hyphen.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Status of the target group.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The type of target group. Valid Values are `IP` | `LAMBDA` | `INSTANCE` | `ALB`
     *
     * The following arguments are optional:
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a TargetGroup resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TargetGroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TargetGroup resources.
 */
export interface TargetGroupState {
    /**
     * ARN of the target group.
     */
    arn?: pulumi.Input<string>;
    /**
     * The target group configuration.
     */
    config?: pulumi.Input<inputs.vpclattice.TargetGroupConfig>;
    /**
     * The name of the target group. The name must be unique within the account. The valid characters are a-z, 0-9, and hyphens (-). You can't use a hyphen as the first or last character, or immediately after another hyphen.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Status of the target group.
     */
    status?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * The type of target group. Valid Values are `IP` | `LAMBDA` | `INSTANCE` | `ALB`
     *
     * The following arguments are optional:
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a TargetGroup resource.
 */
export interface TargetGroupArgs {
    /**
     * The target group configuration.
     */
    config?: pulumi.Input<inputs.vpclattice.TargetGroupConfig>;
    /**
     * The name of the target group. The name must be unique within the account. The valid characters are a-z, 0-9, and hyphens (-). You can't use a hyphen as the first or last character, or immediately after another hyphen.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The type of target group. Valid Values are `IP` | `LAMBDA` | `INSTANCE` | `ALB`
     *
     * The following arguments are optional:
     */
    type: pulumi.Input<string>;
}
