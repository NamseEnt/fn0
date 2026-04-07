import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a AWS Transfer AS2 Profile resource.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.transfer.Profile("example", {
 *     as2Id: "example",
 *     certificateIds: [exampleAwsTransferCertificate.certificateId],
 *     usage: "LOCAL",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Transfer AS2 Profile using the `profile_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:transfer/profile:Profile example p-4221a88afd5f4362a
 * ```
 */
export declare class Profile extends pulumi.CustomResource {
    /**
     * Get an existing Profile resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ProfileState, opts?: pulumi.CustomResourceOptions): Profile;
    /**
     * Returns true if the given object is an instance of Profile.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Profile;
    /**
     * The ARN of the profile.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The As2Id is the AS2 name as defined in the RFC 4130. For inbound ttransfers this is the AS2 From Header for the AS2 messages sent from the partner. For Outbound messages this is the AS2 To Header for the AS2 messages sent to the partner. his ID cannot include spaces.
     */
    readonly as2Id: pulumi.Output<string>;
    /**
     * The list of certificate Ids from the imported certificate operation.
     */
    readonly certificateIds: pulumi.Output<string[] | undefined>;
    /**
     * The unique identifier for the AS2 profile.
     */
    readonly profileId: pulumi.Output<string>;
    /**
     * The profile type should be LOCAL or PARTNER.
     */
    readonly profileType: pulumi.Output<string>;
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
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a Profile resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ProfileArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Profile resources.
 */
export interface ProfileState {
    /**
     * The ARN of the profile.
     */
    arn?: pulumi.Input<string>;
    /**
     * The As2Id is the AS2 name as defined in the RFC 4130. For inbound ttransfers this is the AS2 From Header for the AS2 messages sent from the partner. For Outbound messages this is the AS2 To Header for the AS2 messages sent to the partner. his ID cannot include spaces.
     */
    as2Id?: pulumi.Input<string>;
    /**
     * The list of certificate Ids from the imported certificate operation.
     */
    certificateIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The unique identifier for the AS2 profile.
     */
    profileId?: pulumi.Input<string>;
    /**
     * The profile type should be LOCAL or PARTNER.
     */
    profileType?: pulumi.Input<string>;
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
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a Profile resource.
 */
export interface ProfileArgs {
    /**
     * The As2Id is the AS2 name as defined in the RFC 4130. For inbound ttransfers this is the AS2 From Header for the AS2 messages sent from the partner. For Outbound messages this is the AS2 To Header for the AS2 messages sent to the partner. his ID cannot include spaces.
     */
    as2Id: pulumi.Input<string>;
    /**
     * The list of certificate Ids from the imported certificate operation.
     */
    certificateIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The profile type should be LOCAL or PARTNER.
     */
    profileType: pulumi.Input<string>;
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
}
