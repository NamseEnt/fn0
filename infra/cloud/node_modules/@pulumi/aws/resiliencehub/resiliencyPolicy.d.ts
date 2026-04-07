import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS Resilience Hub Resiliency Policy.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.resiliencehub.ResiliencyPolicy("example", {
 *     name: "testexample",
 *     description: "testexample",
 *     tier: "NonCritical",
 *     dataLocationConstraint: "AnyLocation",
 *     policy: {
 *         region: {
 *             rpo: "24h",
 *             rto: "24h",
 *         },
 *         az: {
 *             rpo: "24h",
 *             rto: "24h",
 *         },
 *         hardware: {
 *             rpo: "24h",
 *             rto: "24h",
 *         },
 *         software: {
 *             rpo: "24h",
 *             rto: "24h",
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Resilience Hub Resiliency Policy using the `arn`. For example:
 *
 * ```sh
 * $ pulumi import aws:resiliencehub/resiliencyPolicy:ResiliencyPolicy example arn:aws:resiliencehub:us-east-1:123456789012:resiliency-policy/8c1cfa29-d1dd-4421-aa68-c9f64cced4c2
 * ```
 */
export declare class ResiliencyPolicy extends pulumi.CustomResource {
    /**
     * Get an existing ResiliencyPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ResiliencyPolicyState, opts?: pulumi.CustomResourceOptions): ResiliencyPolicy;
    /**
     * Returns true if the given object is an instance of ResiliencyPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ResiliencyPolicy;
    /**
     * ARN of the Resiliency Policy.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Data Location Constraint of the Policy.
     * Valid values are `AnyLocation`, `SameContinent`, and `SameCountry`.
     */
    readonly dataLocationConstraint: pulumi.Output<string>;
    /**
     * Description of Resiliency Policy.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Estimated Cost Tier of the Resiliency Policy.
     */
    readonly estimatedCostTier: pulumi.Output<string>;
    /**
     * Name of Resiliency Policy.
     * Must be between 2 and 60 characters long.
     * Must start with an alphanumeric character and contain alphanumeric characters, underscores, or hyphens.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The type of resiliency policy to be created, including the recovery time objective (RTO) and recovery point objective (RPO) in seconds. See `policy`.
     *
     * The following arguments are optional:
     */
    readonly policy: pulumi.Output<outputs.resiliencehub.ResiliencyPolicyPolicy | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
    /**
     * Resiliency Policy Tier.
     * Valid values are `MissionCritical`, `Critical`, `Important`, `CoreServices`, `NonCritical`, and `NotApplicable`.
     */
    readonly tier: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.resiliencehub.ResiliencyPolicyTimeouts | undefined>;
    /**
     * Create a ResiliencyPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ResiliencyPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ResiliencyPolicy resources.
 */
export interface ResiliencyPolicyState {
    /**
     * ARN of the Resiliency Policy.
     */
    arn?: pulumi.Input<string>;
    /**
     * Data Location Constraint of the Policy.
     * Valid values are `AnyLocation`, `SameContinent`, and `SameCountry`.
     */
    dataLocationConstraint?: pulumi.Input<string>;
    /**
     * Description of Resiliency Policy.
     */
    description?: pulumi.Input<string>;
    /**
     * Estimated Cost Tier of the Resiliency Policy.
     */
    estimatedCostTier?: pulumi.Input<string>;
    /**
     * Name of Resiliency Policy.
     * Must be between 2 and 60 characters long.
     * Must start with an alphanumeric character and contain alphanumeric characters, underscores, or hyphens.
     */
    name?: pulumi.Input<string>;
    /**
     * The type of resiliency policy to be created, including the recovery time objective (RTO) and recovery point objective (RPO) in seconds. See `policy`.
     *
     * The following arguments are optional:
     */
    policy?: pulumi.Input<inputs.resiliencehub.ResiliencyPolicyPolicy>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
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
    /**
     * Resiliency Policy Tier.
     * Valid values are `MissionCritical`, `Critical`, `Important`, `CoreServices`, `NonCritical`, and `NotApplicable`.
     */
    tier?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.resiliencehub.ResiliencyPolicyTimeouts>;
}
/**
 * The set of arguments for constructing a ResiliencyPolicy resource.
 */
export interface ResiliencyPolicyArgs {
    /**
     * Data Location Constraint of the Policy.
     * Valid values are `AnyLocation`, `SameContinent`, and `SameCountry`.
     */
    dataLocationConstraint?: pulumi.Input<string>;
    /**
     * Description of Resiliency Policy.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of Resiliency Policy.
     * Must be between 2 and 60 characters long.
     * Must start with an alphanumeric character and contain alphanumeric characters, underscores, or hyphens.
     */
    name?: pulumi.Input<string>;
    /**
     * The type of resiliency policy to be created, including the recovery time objective (RTO) and recovery point objective (RPO) in seconds. See `policy`.
     *
     * The following arguments are optional:
     */
    policy?: pulumi.Input<inputs.resiliencehub.ResiliencyPolicyPolicy>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Resiliency Policy Tier.
     * Valid values are `MissionCritical`, `Critical`, `Important`, `CoreServices`, `NonCritical`, and `NotApplicable`.
     */
    tier: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.resiliencehub.ResiliencyPolicyTimeouts>;
}
