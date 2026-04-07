import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS SESv2 (Simple Email V2) Email Identity.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ### Email Address Identity
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sesv2.EmailIdentity("example", {emailIdentity: "testing@example.com"});
 * ```
 *
 * ### Domain Identity
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sesv2.EmailIdentity("example", {emailIdentity: "example.com"});
 * ```
 *
 * ### Configuration Set
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sesv2.ConfigurationSet("example", {configurationSetName: "example"});
 * const exampleEmailIdentity = new aws.sesv2.EmailIdentity("example", {
 *     emailIdentity: "example.com",
 *     configurationSetName: example.configurationSetName,
 * });
 * ```
 *
 * ### DKIM Signing Attributes (BYODKIM)
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sesv2.EmailIdentity("example", {
 *     emailIdentity: "example.com",
 *     dkimSigningAttributes: {
 *         domainSigningPrivateKey: "MIIJKAIBAAKCAgEA2Se7p8zvnI4yh+Gh9j2rG5e2aRXjg03Y8saiupLnadPH9xvM...",
 *         domainSigningSelector: "example",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SESv2 (Simple Email V2) Email Identity using the `email_identity`. For example:
 *
 * ```sh
 * $ pulumi import aws:sesv2/emailIdentity:EmailIdentity example example.com
 * ```
 */
export declare class EmailIdentity extends pulumi.CustomResource {
    /**
     * Get an existing EmailIdentity resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EmailIdentityState, opts?: pulumi.CustomResourceOptions): EmailIdentity;
    /**
     * Returns true if the given object is an instance of EmailIdentity.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EmailIdentity;
    /**
     * ARN of the Email Identity.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The configuration set to use by default when sending from this identity. Note that any configuration set defined in the email sending request takes precedence.
     */
    readonly configurationSetName: pulumi.Output<string | undefined>;
    /**
     * The configuration of the DKIM authentication settings for an email domain identity.
     */
    readonly dkimSigningAttributes: pulumi.Output<outputs.sesv2.EmailIdentityDkimSigningAttributes>;
    /**
     * The email address or domain to verify.
     *
     * The following arguments are optional:
     */
    readonly emailIdentity: pulumi.Output<string>;
    /**
     * The email identity type. Valid values: `EMAIL_ADDRESS`, `DOMAIN`.
     */
    readonly identityType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
     * The verification status of the identity. The status can be one of the following: `PENDING`, `SUCCESS`, `FAILED`, `TEMPORARY_FAILURE`, and `NOT_STARTED`.
     */
    readonly verificationStatus: pulumi.Output<string>;
    /**
     * Specifies whether or not the identity is verified.
     */
    readonly verifiedForSendingStatus: pulumi.Output<boolean>;
    /**
     * Create a EmailIdentity resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EmailIdentityArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EmailIdentity resources.
 */
export interface EmailIdentityState {
    /**
     * ARN of the Email Identity.
     */
    arn?: pulumi.Input<string>;
    /**
     * The configuration set to use by default when sending from this identity. Note that any configuration set defined in the email sending request takes precedence.
     */
    configurationSetName?: pulumi.Input<string>;
    /**
     * The configuration of the DKIM authentication settings for an email domain identity.
     */
    dkimSigningAttributes?: pulumi.Input<inputs.sesv2.EmailIdentityDkimSigningAttributes>;
    /**
     * The email address or domain to verify.
     *
     * The following arguments are optional:
     */
    emailIdentity?: pulumi.Input<string>;
    /**
     * The email identity type. Valid values: `EMAIL_ADDRESS`, `DOMAIN`.
     */
    identityType?: pulumi.Input<string>;
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
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The verification status of the identity. The status can be one of the following: `PENDING`, `SUCCESS`, `FAILED`, `TEMPORARY_FAILURE`, and `NOT_STARTED`.
     */
    verificationStatus?: pulumi.Input<string>;
    /**
     * Specifies whether or not the identity is verified.
     */
    verifiedForSendingStatus?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a EmailIdentity resource.
 */
export interface EmailIdentityArgs {
    /**
     * The configuration set to use by default when sending from this identity. Note that any configuration set defined in the email sending request takes precedence.
     */
    configurationSetName?: pulumi.Input<string>;
    /**
     * The configuration of the DKIM authentication settings for an email domain identity.
     */
    dkimSigningAttributes?: pulumi.Input<inputs.sesv2.EmailIdentityDkimSigningAttributes>;
    /**
     * The email address or domain to verify.
     *
     * The following arguments are optional:
     */
    emailIdentity: pulumi.Input<string>;
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
}
