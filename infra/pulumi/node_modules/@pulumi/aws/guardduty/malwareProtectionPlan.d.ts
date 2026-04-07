import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage a GuardDuty malware protection plan.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.guardduty.MalwareProtectionPlan("example", {
 *     role: exampleAwsIamRole.arn,
 *     protectedResource: {
 *         s3Bucket: {
 *             bucketName: exampleAwsS3Bucket.id,
 *             objectPrefixes: [
 *                 "example1",
 *                 "example2",
 *             ],
 *         },
 *     },
 *     actions: [{
 *         tagging: [{
 *             status: "ENABLED",
 *         }],
 *     }],
 *     tags: {
 *         Name: "example",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import GuardDuty malware protection plans using their IDs. For example:
 *
 * ```sh
 * $ pulumi import aws:guardduty/malwareProtectionPlan:MalwareProtectionPlan example 1234567890abcdef0123
 * ```
 */
export declare class MalwareProtectionPlan extends pulumi.CustomResource {
    /**
     * Get an existing MalwareProtectionPlan resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MalwareProtectionPlanState, opts?: pulumi.CustomResourceOptions): MalwareProtectionPlan;
    /**
     * Returns true if the given object is an instance of MalwareProtectionPlan.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is MalwareProtectionPlan;
    /**
     * Information about whether the tags will be added to the S3 object after scanning. See `actions` below.
     */
    readonly actions: pulumi.Output<outputs.guardduty.MalwareProtectionPlanAction[]>;
    /**
     * The ARN of the GuardDuty malware protection plan
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The timestamp when the Malware Protection plan resource was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Information about the protected resource that is associated with the created Malware Protection plan. Presently, S3Bucket is the only supported protected resource. See `protectedResource` below.
     */
    readonly protectedResource: pulumi.Output<outputs.guardduty.MalwareProtectionPlanProtectedResource | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * ARN of IAM role that includes the permissions required to scan and add tags to the associated protected resource.
     */
    readonly role: pulumi.Output<string>;
    /**
     * The GuardDuty malware protection plan status. Valid values are `ACTIVE`, `WARNING`, and `ERROR`.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a MalwareProtectionPlan resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MalwareProtectionPlanArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering MalwareProtectionPlan resources.
 */
export interface MalwareProtectionPlanState {
    /**
     * Information about whether the tags will be added to the S3 object after scanning. See `actions` below.
     */
    actions?: pulumi.Input<pulumi.Input<inputs.guardduty.MalwareProtectionPlanAction>[]>;
    /**
     * The ARN of the GuardDuty malware protection plan
     */
    arn?: pulumi.Input<string>;
    /**
     * The timestamp when the Malware Protection plan resource was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Information about the protected resource that is associated with the created Malware Protection plan. Presently, S3Bucket is the only supported protected resource. See `protectedResource` below.
     */
    protectedResource?: pulumi.Input<inputs.guardduty.MalwareProtectionPlanProtectedResource>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of IAM role that includes the permissions required to scan and add tags to the associated protected resource.
     */
    role?: pulumi.Input<string>;
    /**
     * The GuardDuty malware protection plan status. Valid values are `ACTIVE`, `WARNING`, and `ERROR`.
     */
    status?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a MalwareProtectionPlan resource.
 */
export interface MalwareProtectionPlanArgs {
    /**
     * Information about whether the tags will be added to the S3 object after scanning. See `actions` below.
     */
    actions?: pulumi.Input<pulumi.Input<inputs.guardduty.MalwareProtectionPlanAction>[]>;
    /**
     * Information about the protected resource that is associated with the created Malware Protection plan. Presently, S3Bucket is the only supported protected resource. See `protectedResource` below.
     */
    protectedResource?: pulumi.Input<inputs.guardduty.MalwareProtectionPlanProtectedResource>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ARN of IAM role that includes the permissions required to scan and add tags to the associated protected resource.
     */
    role: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
