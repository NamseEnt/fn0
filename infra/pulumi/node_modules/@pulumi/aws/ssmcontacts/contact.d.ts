import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS SSM Contact.
 *
 * > **NOTE:** A contact implicitly depends on a replication set. If you configured your replication set in Pulumi, we recommend you add it to the `dependsOn` argument for the Contact Resource.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ssmcontacts.Contact("example", {
 *     alias: "alias",
 *     type: "PERSONAL",
 * }, {
 *     dependsOn: [exampleAwsSsmincidentsReplicationSet],
 * });
 * ```
 *
 * ### Usage With All Fields
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ssmcontacts.Contact("example", {
 *     alias: "alias",
 *     displayName: "displayName",
 *     type: "ESCALATION",
 *     tags: {
 *         key: "value",
 *     },
 * }, {
 *     dependsOn: [exampleAwsSsmincidentsReplicationSet],
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the contact.
 *
 * Using `pulumi import`, import SSM Contact using the `ARN`. For example:
 *
 * console
 *
 * % pulumi import aws_ssmcontacts_contact.example {ARNValue}
 */
export declare class Contact extends pulumi.CustomResource {
    /**
     * Get an existing Contact resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ContactState, opts?: pulumi.CustomResourceOptions): Contact;
    /**
     * Returns true if the given object is an instance of Contact.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Contact;
    /**
     * A unique and identifiable alias for the contact or escalation plan. Must be between 1 and 255 characters, and may contain alphanumerics, underscores (`_`), and hyphens (`-`).
     */
    readonly alias: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the contact or escalation plan.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Full friendly name of the contact or escalation plan. If set, must be between 1 and 255 characters, and may contain alphanumerics, underscores (`_`), hyphens (`-`), periods (`.`), and spaces.
     */
    readonly displayName: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value tags for the monitor. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The type of contact engaged. A single contact is type PERSONAL and an escalation
     * plan is type ESCALATION.
     *
     * The following arguments are optional:
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a Contact resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ContactArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Contact resources.
 */
export interface ContactState {
    /**
     * A unique and identifiable alias for the contact or escalation plan. Must be between 1 and 255 characters, and may contain alphanumerics, underscores (`_`), and hyphens (`-`).
     */
    alias?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the contact or escalation plan.
     */
    arn?: pulumi.Input<string>;
    /**
     * Full friendly name of the contact or escalation plan. If set, must be between 1 and 255 characters, and may contain alphanumerics, underscores (`_`), hyphens (`-`), periods (`.`), and spaces.
     */
    displayName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value tags for the monitor. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The type of contact engaged. A single contact is type PERSONAL and an escalation
     * plan is type ESCALATION.
     *
     * The following arguments are optional:
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Contact resource.
 */
export interface ContactArgs {
    /**
     * A unique and identifiable alias for the contact or escalation plan. Must be between 1 and 255 characters, and may contain alphanumerics, underscores (`_`), and hyphens (`-`).
     */
    alias: pulumi.Input<string>;
    /**
     * Full friendly name of the contact or escalation plan. If set, must be between 1 and 255 characters, and may contain alphanumerics, underscores (`_`), hyphens (`-`), periods (`.`), and spaces.
     */
    displayName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value tags for the monitor. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The type of contact engaged. A single contact is type PERSONAL and an escalation
     * plan is type ESCALATION.
     *
     * The following arguments are optional:
     */
    type: pulumi.Input<string>;
}
