import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing a Verified Access Group.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.verifiedaccess.Group("example", {verifiedaccessInstanceId: exampleAwsVerifiedaccessInstance.id});
 * ```
 *
 * ### Usage with KMS Key
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testKey = new aws.kms.Key("test_key", {description: "KMS key for Verified Access Group test"});
 * const test = new aws.verifiedaccess.Group("test", {
 *     verifiedaccessInstanceId: testAwsVerifiedaccessInstanceTrustProviderAttachment.verifiedaccessInstanceId,
 *     sseConfiguration: {
 *         kmsKeyArn: testKey.arn,
 *     },
 * });
 * ```
 */
export declare class Group extends pulumi.CustomResource {
    /**
     * Get an existing Group resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: GroupState, opts?: pulumi.CustomResourceOptions): Group;
    /**
     * Returns true if the given object is an instance of Group.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Group;
    /**
     * Timestamp when the access group was created.
     */
    readonly creationTime: pulumi.Output<string>;
    /**
     * Timestamp when the access group was deleted.
     */
    readonly deletionTime: pulumi.Output<string>;
    /**
     * Description of the verified access group.
     */
    readonly description: pulumi.Output<string>;
    /**
     * Timestamp when the access group was last updated.
     */
    readonly lastUpdatedTime: pulumi.Output<string>;
    /**
     * AWS account number owning this resource.
     */
    readonly owner: pulumi.Output<string>;
    /**
     * The policy document that is associated with this resource.
     */
    readonly policyDocument: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Configuration block to use KMS keys for server-side encryption.
     */
    readonly sseConfiguration: pulumi.Output<outputs.verifiedaccess.GroupSseConfiguration>;
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
     * ARN of this verified acess group.
     */
    readonly verifiedaccessGroupArn: pulumi.Output<string>;
    /**
     * ID of this verified access group.
     */
    readonly verifiedaccessGroupId: pulumi.Output<string>;
    /**
     * The id of the verified access instance this group is associated with.
     *
     * The following arguments are optional:
     */
    readonly verifiedaccessInstanceId: pulumi.Output<string>;
    /**
     * Create a Group resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: GroupArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Group resources.
 */
export interface GroupState {
    /**
     * Timestamp when the access group was created.
     */
    creationTime?: pulumi.Input<string>;
    /**
     * Timestamp when the access group was deleted.
     */
    deletionTime?: pulumi.Input<string>;
    /**
     * Description of the verified access group.
     */
    description?: pulumi.Input<string>;
    /**
     * Timestamp when the access group was last updated.
     */
    lastUpdatedTime?: pulumi.Input<string>;
    /**
     * AWS account number owning this resource.
     */
    owner?: pulumi.Input<string>;
    /**
     * The policy document that is associated with this resource.
     */
    policyDocument?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block to use KMS keys for server-side encryption.
     */
    sseConfiguration?: pulumi.Input<inputs.verifiedaccess.GroupSseConfiguration>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * ARN of this verified acess group.
     */
    verifiedaccessGroupArn?: pulumi.Input<string>;
    /**
     * ID of this verified access group.
     */
    verifiedaccessGroupId?: pulumi.Input<string>;
    /**
     * The id of the verified access instance this group is associated with.
     *
     * The following arguments are optional:
     */
    verifiedaccessInstanceId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Group resource.
 */
export interface GroupArgs {
    /**
     * Description of the verified access group.
     */
    description?: pulumi.Input<string>;
    /**
     * The policy document that is associated with this resource.
     */
    policyDocument?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Configuration block to use KMS keys for server-side encryption.
     */
    sseConfiguration?: pulumi.Input<inputs.verifiedaccess.GroupSseConfiguration>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The id of the verified access instance this group is associated with.
     *
     * The following arguments are optional:
     */
    verifiedaccessInstanceId: pulumi.Input<string>;
}
