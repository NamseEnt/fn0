import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS SESv2 (Simple Email V2) Contact List.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sesv2.ContactList("example", {contactListName: "example"});
 * ```
 *
 * ### Extended Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sesv2.ContactList("example", {
 *     contactListName: "example",
 *     description: "description",
 *     topics: [{
 *         defaultSubscriptionStatus: "OPT_IN",
 *         description: "topic description",
 *         displayName: "Example Topic",
 *         topicName: "example-topic",
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SESv2 (Simple Email V2) Contact List using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:sesv2/contactList:ContactList example example
 * ```
 */
export declare class ContactList extends pulumi.CustomResource {
    /**
     * Get an existing ContactList resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ContactListState, opts?: pulumi.CustomResourceOptions): ContactList;
    /**
     * Returns true if the given object is an instance of ContactList.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ContactList;
    readonly arn: pulumi.Output<string>;
    /**
     * Name of the contact list.
     *
     * The following arguments are optional:
     */
    readonly contactListName: pulumi.Output<string>;
    /**
     * Timestamp noting when the contact list was created in ISO 8601 format.
     */
    readonly createdTimestamp: pulumi.Output<string>;
    /**
     * Description of what the contact list is about.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Timestamp noting the last time the contact list was updated in ISO 8601 format.
     */
    readonly lastUpdatedTimestamp: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of resource tags for the contact list. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Configuration block(s) with topic for the contact list. Detailed below.
     */
    readonly topics: pulumi.Output<outputs.sesv2.ContactListTopic[] | undefined>;
    /**
     * Create a ContactList resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ContactListArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ContactList resources.
 */
export interface ContactListState {
    arn?: pulumi.Input<string>;
    /**
     * Name of the contact list.
     *
     * The following arguments are optional:
     */
    contactListName?: pulumi.Input<string>;
    /**
     * Timestamp noting when the contact list was created in ISO 8601 format.
     */
    createdTimestamp?: pulumi.Input<string>;
    /**
     * Description of what the contact list is about.
     */
    description?: pulumi.Input<string>;
    /**
     * Timestamp noting the last time the contact list was updated in ISO 8601 format.
     */
    lastUpdatedTimestamp?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags for the contact list. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block(s) with topic for the contact list. Detailed below.
     */
    topics?: pulumi.Input<pulumi.Input<inputs.sesv2.ContactListTopic>[]>;
}
/**
 * The set of arguments for constructing a ContactList resource.
 */
export interface ContactListArgs {
    /**
     * Name of the contact list.
     *
     * The following arguments are optional:
     */
    contactListName: pulumi.Input<string>;
    /**
     * Description of what the contact list is about.
     */
    description?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags for the contact list. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Configuration block(s) with topic for the contact list. Detailed below.
     */
    topics?: pulumi.Input<pulumi.Input<inputs.sesv2.ContactListTopic>[]>;
}
