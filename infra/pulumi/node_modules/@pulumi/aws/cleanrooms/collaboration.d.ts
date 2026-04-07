import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a AWS Clean Rooms collaboration.
 * All members included in the definition will be invited to join the collaboration and can create memberships.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const testCollaboration = new aws.cleanrooms.Collaboration("test_collaboration", {
 *     name: "pulumi-example-collaboration",
 *     creatorMemberAbilities: [
 *         "CAN_QUERY",
 *         "CAN_RECEIVE_RESULTS",
 *     ],
 *     creatorDisplayName: "Creator ",
 *     description: "I made this collaboration with Pulumi!",
 *     queryLogStatus: "DISABLED",
 *     analyticsEngine: "SPARK",
 *     dataEncryptionMetadata: {
 *         allowClearText: true,
 *         allowDuplicates: true,
 *         allowJoinsOnColumnsWithDifferentNames: true,
 *         preserveNulls: false,
 *     },
 *     members: [{
 *         accountId: "123456789012",
 *         displayName: "Other member",
 *         memberAbilities: [],
 *     }],
 *     tags: {
 *         Project: "Pulumi",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_cleanrooms_collaboration` using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:cleanrooms/collaboration:Collaboration collaboration 1234abcd-12ab-34cd-56ef-1234567890ab
 * ```
 */
export declare class Collaboration extends pulumi.CustomResource {
    /**
     * Get an existing Collaboration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CollaborationState, opts?: pulumi.CustomResourceOptions): Collaboration;
    /**
     * Returns true if the given object is an instance of Collaboration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Collaboration;
    /**
     * Analytics engine used by the collaboration. Valid values are `CLEAN_ROOMS_SQL` (deprecated) and `SPARK`.
     */
    readonly analyticsEngine: pulumi.Output<string | undefined>;
    /**
     * ARN of the collaboration.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Date and time the collaboration was created.
     * * `member status` - For each member included in the collaboration an additional computed attribute of status is added. These values [may be found here](https://docs.aws.amazon.com/clean-rooms/latest/apireference/API_MemberSummary.html#API-Type-MemberSummary-status).
     */
    readonly createTime: pulumi.Output<string>;
    /**
     * The name for the member record for the collaboration creator.
     */
    readonly creatorDisplayName: pulumi.Output<string>;
    /**
     * The list of member abilities for the creator of the collaboration.  Valid values [may be found here](https://docs.aws.amazon.com/clean-rooms/latest/apireference/API_CreateCollaboration.html#API-CreateCollaboration-request-creatorMemberAbilities).
     */
    readonly creatorMemberAbilities: pulumi.Output<string[]>;
    /**
     * a collection of settings which determine how the [c3r client](https://docs.aws.amazon.com/clean-rooms/latest/userguide/crypto-computing.html) will encrypt data for use within this collaboration.
     * * `data_encryption_metadata.allow_clear_text` - (Required - Forces new resource) - Indicates whether encrypted tables can contain cleartext data. This is a boolea
     * field.
     * * `data_encryption_metadata.allow_duplicates` - (Required - Forces new resource ) - Indicates whether Fingerprint columns can contain duplicate entries. This is a
     * boolean field.
     * * `data_encryption_metadata.allow_joins_on_columns_with_different_names` - (Required - Forces new resource) - Indicates whether Fingerprint columns can be joined
     * n any other Fingerprint column with a different name. This is a boolean field.
     * * `data_encryption_metadata.preserve_nulls` - (Required - Forces new resource) - Indicates whether NULL values are to be copied as NULL to encrypted tables (true)
     * or cryptographically processed (false).
     */
    readonly dataEncryptionMetadata: pulumi.Output<outputs.cleanrooms.CollaborationDataEncryptionMetadata | undefined>;
    /**
     * A description for a collaboration.
     */
    readonly description: pulumi.Output<string>;
    /**
     * Additional members of the collaboration which will be invited to join the collaboration.
     * * `member.account_id` - (Required - Forces new resource) - The account id for the invited member.
     * * `member.display_name` - (Required - Forces new resource) - The display name for the invited member.
     * * `member.member_abilities` - (Required - Forces new resource) - The list of abilities for the invited member. Valid values [may be found here](https://docs.aws.amazon.com/clean-rooms/latest/apireference/API_CreateCollaboration.html#API-CreateCollaboration-request-creatorMemberAbilities).
     */
    readonly members: pulumi.Output<outputs.cleanrooms.CollaborationMember[] | undefined>;
    /**
     * The name of the collaboration.  Collaboration names do not need to be unique.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Determines if members of the collaboration can enable query logs within their own.
     * emberships. Valid values [may be found here](https://docs.aws.amazon.com/clean-rooms/latest/apireference/API_CreateCollaboration.html#API-CreateCollaboration-request-queryLogStatus).
     *
     * The following arguments are optional:
     */
    readonly queryLogStatus: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key value pairs which tag the collaboration.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly updateTime: pulumi.Output<string>;
    /**
     * Create a Collaboration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CollaborationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Collaboration resources.
 */
export interface CollaborationState {
    /**
     * Analytics engine used by the collaboration. Valid values are `CLEAN_ROOMS_SQL` (deprecated) and `SPARK`.
     */
    analyticsEngine?: pulumi.Input<string>;
    /**
     * ARN of the collaboration.
     */
    arn?: pulumi.Input<string>;
    /**
     * Date and time the collaboration was created.
     * * `member status` - For each member included in the collaboration an additional computed attribute of status is added. These values [may be found here](https://docs.aws.amazon.com/clean-rooms/latest/apireference/API_MemberSummary.html#API-Type-MemberSummary-status).
     */
    createTime?: pulumi.Input<string>;
    /**
     * The name for the member record for the collaboration creator.
     */
    creatorDisplayName?: pulumi.Input<string>;
    /**
     * The list of member abilities for the creator of the collaboration.  Valid values [may be found here](https://docs.aws.amazon.com/clean-rooms/latest/apireference/API_CreateCollaboration.html#API-CreateCollaboration-request-creatorMemberAbilities).
     */
    creatorMemberAbilities?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * a collection of settings which determine how the [c3r client](https://docs.aws.amazon.com/clean-rooms/latest/userguide/crypto-computing.html) will encrypt data for use within this collaboration.
     * * `data_encryption_metadata.allow_clear_text` - (Required - Forces new resource) - Indicates whether encrypted tables can contain cleartext data. This is a boolea
     * field.
     * * `data_encryption_metadata.allow_duplicates` - (Required - Forces new resource ) - Indicates whether Fingerprint columns can contain duplicate entries. This is a
     * boolean field.
     * * `data_encryption_metadata.allow_joins_on_columns_with_different_names` - (Required - Forces new resource) - Indicates whether Fingerprint columns can be joined
     * n any other Fingerprint column with a different name. This is a boolean field.
     * * `data_encryption_metadata.preserve_nulls` - (Required - Forces new resource) - Indicates whether NULL values are to be copied as NULL to encrypted tables (true)
     * or cryptographically processed (false).
     */
    dataEncryptionMetadata?: pulumi.Input<inputs.cleanrooms.CollaborationDataEncryptionMetadata>;
    /**
     * A description for a collaboration.
     */
    description?: pulumi.Input<string>;
    /**
     * Additional members of the collaboration which will be invited to join the collaboration.
     * * `member.account_id` - (Required - Forces new resource) - The account id for the invited member.
     * * `member.display_name` - (Required - Forces new resource) - The display name for the invited member.
     * * `member.member_abilities` - (Required - Forces new resource) - The list of abilities for the invited member. Valid values [may be found here](https://docs.aws.amazon.com/clean-rooms/latest/apireference/API_CreateCollaboration.html#API-CreateCollaboration-request-creatorMemberAbilities).
     */
    members?: pulumi.Input<pulumi.Input<inputs.cleanrooms.CollaborationMember>[]>;
    /**
     * The name of the collaboration.  Collaboration names do not need to be unique.
     */
    name?: pulumi.Input<string>;
    /**
     * Determines if members of the collaboration can enable query logs within their own.
     * emberships. Valid values [may be found here](https://docs.aws.amazon.com/clean-rooms/latest/apireference/API_CreateCollaboration.html#API-CreateCollaboration-request-queryLogStatus).
     *
     * The following arguments are optional:
     */
    queryLogStatus?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key value pairs which tag the collaboration.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    updateTime?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Collaboration resource.
 */
export interface CollaborationArgs {
    /**
     * Analytics engine used by the collaboration. Valid values are `CLEAN_ROOMS_SQL` (deprecated) and `SPARK`.
     */
    analyticsEngine?: pulumi.Input<string>;
    /**
     * The name for the member record for the collaboration creator.
     */
    creatorDisplayName: pulumi.Input<string>;
    /**
     * The list of member abilities for the creator of the collaboration.  Valid values [may be found here](https://docs.aws.amazon.com/clean-rooms/latest/apireference/API_CreateCollaboration.html#API-CreateCollaboration-request-creatorMemberAbilities).
     */
    creatorMemberAbilities: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * a collection of settings which determine how the [c3r client](https://docs.aws.amazon.com/clean-rooms/latest/userguide/crypto-computing.html) will encrypt data for use within this collaboration.
     * * `data_encryption_metadata.allow_clear_text` - (Required - Forces new resource) - Indicates whether encrypted tables can contain cleartext data. This is a boolea
     * field.
     * * `data_encryption_metadata.allow_duplicates` - (Required - Forces new resource ) - Indicates whether Fingerprint columns can contain duplicate entries. This is a
     * boolean field.
     * * `data_encryption_metadata.allow_joins_on_columns_with_different_names` - (Required - Forces new resource) - Indicates whether Fingerprint columns can be joined
     * n any other Fingerprint column with a different name. This is a boolean field.
     * * `data_encryption_metadata.preserve_nulls` - (Required - Forces new resource) - Indicates whether NULL values are to be copied as NULL to encrypted tables (true)
     * or cryptographically processed (false).
     */
    dataEncryptionMetadata?: pulumi.Input<inputs.cleanrooms.CollaborationDataEncryptionMetadata>;
    /**
     * A description for a collaboration.
     */
    description: pulumi.Input<string>;
    /**
     * Additional members of the collaboration which will be invited to join the collaboration.
     * * `member.account_id` - (Required - Forces new resource) - The account id for the invited member.
     * * `member.display_name` - (Required - Forces new resource) - The display name for the invited member.
     * * `member.member_abilities` - (Required - Forces new resource) - The list of abilities for the invited member. Valid values [may be found here](https://docs.aws.amazon.com/clean-rooms/latest/apireference/API_CreateCollaboration.html#API-CreateCollaboration-request-creatorMemberAbilities).
     */
    members?: pulumi.Input<pulumi.Input<inputs.cleanrooms.CollaborationMember>[]>;
    /**
     * The name of the collaboration.  Collaboration names do not need to be unique.
     */
    name?: pulumi.Input<string>;
    /**
     * Determines if members of the collaboration can enable query logs within their own.
     * emberships. Valid values [may be found here](https://docs.aws.amazon.com/clean-rooms/latest/apireference/API_CreateCollaboration.html#API-CreateCollaboration-request-queryLogStatus).
     *
     * The following arguments are optional:
     */
    queryLogStatus: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key value pairs which tag the collaboration.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
