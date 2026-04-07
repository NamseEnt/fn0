import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a AWS Clean Rooms membership. Memberships are used to join a Clean Rooms collaboration by the invited member.
 *
 * ## Example Usage
 *
 * ### Membership with tags
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testMembership = new aws.cleanrooms.Membership("test_membership", {
 *     collaborationId: "1234abcd-12ab-34cd-56ef-1234567890ab",
 *     queryLogStatus: "DISABLED",
 *     defaultResultConfiguration: {
 *         roleArn: "arn:aws:iam::123456789012:role/role-name",
 *         outputConfiguration: {
 *             s3: {
 *                 bucket: "test-bucket",
 *                 resultFormat: "PARQUET",
 *                 keyPrefix: "test-prefix",
 *             },
 *         },
 *     },
 *     tags: {
 *         Project: "Terraform",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_cleanrooms_membership` using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:cleanrooms/membership:Membership membership 1234abcd-12ab-34cd-56ef-1234567890ab
 * ```
 */
export declare class Membership extends pulumi.CustomResource {
    /**
     * Get an existing Membership resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: MembershipState, opts?: pulumi.CustomResourceOptions): Membership;
    /**
     * Returns true if the given object is an instance of Membership.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Membership;
    /**
     * The ARN of the membership.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The ARN of the joined collaboration.
     */
    readonly collaborationArn: pulumi.Output<string>;
    /**
     * The account ID of the collaboration's creator.
     */
    readonly collaborationCreatorAccountId: pulumi.Output<string>;
    /**
     * The display name of the collaboration's creator.
     */
    readonly collaborationCreatorDisplayName: pulumi.Output<string>;
    /**
     * The ID of the collaboration to which the member was invited.
     */
    readonly collaborationId: pulumi.Output<string>;
    /**
     * The name of the joined collaboration.
     */
    readonly collaborationName: pulumi.Output<string>;
    /**
     * The date and time the membership was created.
     */
    readonly createTime: pulumi.Output<string>;
    /**
     * The default configuration for a query result.
     */
    readonly defaultResultConfiguration: pulumi.Output<outputs.cleanrooms.MembershipDefaultResultConfiguration | undefined>;
    /**
     * The list of abilities for the invited member.
     */
    readonly memberAbilities: pulumi.Output<string[]>;
    readonly paymentConfiguration: pulumi.Output<outputs.cleanrooms.MembershipPaymentConfiguration | undefined>;
    /**
     * An indicator as to whether query logging has been enabled or disabled for the membership.
     */
    readonly queryLogStatus: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The status of the membership.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Key value pairs which tag the membership.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The date and time the membership was last updated.
     */
    readonly updateTime: pulumi.Output<string>;
    /**
     * Create a Membership resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: MembershipArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Membership resources.
 */
export interface MembershipState {
    /**
     * The ARN of the membership.
     */
    arn?: pulumi.Input<string>;
    /**
     * The ARN of the joined collaboration.
     */
    collaborationArn?: pulumi.Input<string>;
    /**
     * The account ID of the collaboration's creator.
     */
    collaborationCreatorAccountId?: pulumi.Input<string>;
    /**
     * The display name of the collaboration's creator.
     */
    collaborationCreatorDisplayName?: pulumi.Input<string>;
    /**
     * The ID of the collaboration to which the member was invited.
     */
    collaborationId?: pulumi.Input<string>;
    /**
     * The name of the joined collaboration.
     */
    collaborationName?: pulumi.Input<string>;
    /**
     * The date and time the membership was created.
     */
    createTime?: pulumi.Input<string>;
    /**
     * The default configuration for a query result.
     */
    defaultResultConfiguration?: pulumi.Input<inputs.cleanrooms.MembershipDefaultResultConfiguration>;
    /**
     * The list of abilities for the invited member.
     */
    memberAbilities?: pulumi.Input<pulumi.Input<string>[]>;
    paymentConfiguration?: pulumi.Input<inputs.cleanrooms.MembershipPaymentConfiguration>;
    /**
     * An indicator as to whether query logging has been enabled or disabled for the membership.
     */
    queryLogStatus?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The status of the membership.
     */
    status?: pulumi.Input<string>;
    /**
     * Key value pairs which tag the membership.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The date and time the membership was last updated.
     */
    updateTime?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Membership resource.
 */
export interface MembershipArgs {
    /**
     * The ID of the collaboration to which the member was invited.
     */
    collaborationId: pulumi.Input<string>;
    /**
     * The default configuration for a query result.
     */
    defaultResultConfiguration?: pulumi.Input<inputs.cleanrooms.MembershipDefaultResultConfiguration>;
    paymentConfiguration?: pulumi.Input<inputs.cleanrooms.MembershipPaymentConfiguration>;
    /**
     * An indicator as to whether query logging has been enabled or disabled for the membership.
     */
    queryLogStatus: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key value pairs which tag the membership.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
