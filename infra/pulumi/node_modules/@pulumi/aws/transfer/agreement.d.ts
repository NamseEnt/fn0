import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a AWS Transfer AS2 Agreement resource.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.transfer.Agreement("example", {
 *     accessRole: test.arn,
 *     baseDirectory: "/DOC-EXAMPLE-BUCKET/home/mydirectory",
 *     description: "example",
 *     localProfileId: local.profileId,
 *     partnerProfileId: partner.profileId,
 *     serverId: testAwsTransferServer.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Transfer AS2 Agreement using the `server_id/agreement_id`. For example:
 *
 * ```sh
 * $ pulumi import aws:transfer/agreement:Agreement example s-4221a88afd5f4362a/a-4221a88afd5f4362a
 * ```
 */
export declare class Agreement extends pulumi.CustomResource {
    /**
     * Get an existing Agreement resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AgreementState, opts?: pulumi.CustomResourceOptions): Agreement;
    /**
     * Returns true if the given object is an instance of Agreement.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Agreement;
    /**
     * The IAM Role which provides read and write access to the parent directory of the file location mentioned in the StartFileTransfer request.
     */
    readonly accessRole: pulumi.Output<string>;
    /**
     * The unique identifier for the AS2 agreement.
     */
    readonly agreementId: pulumi.Output<string>;
    /**
     * The ARN of the agreement.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The landing directory for the files transferred by using the AS2 protocol.
     */
    readonly baseDirectory: pulumi.Output<string>;
    /**
     * The Optional description of the transdfer.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The unique identifier for the AS2 local profile.
     */
    readonly localProfileId: pulumi.Output<string>;
    /**
     * The unique identifier for the AS2 partner profile.
     */
    readonly partnerProfileId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The unique server identifier for the server instance. This is the specific server the agreement uses.
     */
    readonly serverId: pulumi.Output<string>;
    readonly status: pulumi.Output<string>;
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
     * Create a Agreement resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AgreementArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Agreement resources.
 */
export interface AgreementState {
    /**
     * The IAM Role which provides read and write access to the parent directory of the file location mentioned in the StartFileTransfer request.
     */
    accessRole?: pulumi.Input<string>;
    /**
     * The unique identifier for the AS2 agreement.
     */
    agreementId?: pulumi.Input<string>;
    /**
     * The ARN of the agreement.
     */
    arn?: pulumi.Input<string>;
    /**
     * The landing directory for the files transferred by using the AS2 protocol.
     */
    baseDirectory?: pulumi.Input<string>;
    /**
     * The Optional description of the transdfer.
     */
    description?: pulumi.Input<string>;
    /**
     * The unique identifier for the AS2 local profile.
     */
    localProfileId?: pulumi.Input<string>;
    /**
     * The unique identifier for the AS2 partner profile.
     */
    partnerProfileId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The unique server identifier for the server instance. This is the specific server the agreement uses.
     */
    serverId?: pulumi.Input<string>;
    status?: pulumi.Input<string>;
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
 * The set of arguments for constructing a Agreement resource.
 */
export interface AgreementArgs {
    /**
     * The IAM Role which provides read and write access to the parent directory of the file location mentioned in the StartFileTransfer request.
     */
    accessRole: pulumi.Input<string>;
    /**
     * The landing directory for the files transferred by using the AS2 protocol.
     */
    baseDirectory: pulumi.Input<string>;
    /**
     * The Optional description of the transdfer.
     */
    description?: pulumi.Input<string>;
    /**
     * The unique identifier for the AS2 local profile.
     */
    localProfileId: pulumi.Input<string>;
    /**
     * The unique identifier for the AS2 partner profile.
     */
    partnerProfileId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The unique server identifier for the server instance. This is the specific server the agreement uses.
     */
    serverId: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
