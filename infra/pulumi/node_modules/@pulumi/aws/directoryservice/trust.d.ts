import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a trust relationship between two Active Directory Directories.
 *
 * The directories may either be both AWS Managed Microsoft AD domains or an AWS Managed Microsoft AD domain and a self-managed Active Directory Domain.
 *
 * The Trust relationship must be configured on both sides of the relationship.
 * If a Trust has only been created on one side, it will be in the state `VerifyFailed`.
 * Once the second Trust is created, the first will update to the correct state.
 *
 * ## Example Usage
 *
 * ### Two-Way Trust
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const oneDirectory = new aws.directoryservice.Directory("one", {
 *     name: "one.example.com",
 *     type: "MicrosoftAD",
 * });
 * const twoDirectory = new aws.directoryservice.Directory("two", {
 *     name: "two.example.com",
 *     type: "MicrosoftAD",
 * });
 * const one = new aws.directoryservice.Trust("one", {
 *     directoryId: oneDirectory.id,
 *     remoteDomainName: twoDirectory.name,
 *     trustDirection: "Two-Way",
 *     trustPassword: "Some0therPassword",
 *     conditionalForwarderIpAddrs: twoDirectory.dnsIpAddresses,
 * });
 * const two = new aws.directoryservice.Trust("two", {
 *     directoryId: twoDirectory.id,
 *     remoteDomainName: oneDirectory.name,
 *     trustDirection: "Two-Way",
 *     trustPassword: "Some0therPassword",
 *     conditionalForwarderIpAddrs: oneDirectory.dnsIpAddresses,
 * });
 * ```
 *
 * ### One-Way Trust
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const oneDirectory = new aws.directoryservice.Directory("one", {
 *     name: "one.example.com",
 *     type: "MicrosoftAD",
 * });
 * const twoDirectory = new aws.directoryservice.Directory("two", {
 *     name: "two.example.com",
 *     type: "MicrosoftAD",
 * });
 * const one = new aws.directoryservice.Trust("one", {
 *     directoryId: oneDirectory.id,
 *     remoteDomainName: twoDirectory.name,
 *     trustDirection: "One-Way: Incoming",
 *     trustPassword: "Some0therPassword",
 *     conditionalForwarderIpAddrs: twoDirectory.dnsIpAddresses,
 * });
 * const two = new aws.directoryservice.Trust("two", {
 *     directoryId: twoDirectory.id,
 *     remoteDomainName: oneDirectory.name,
 *     trustDirection: "One-Way: Outgoing",
 *     trustPassword: "Some0therPassword",
 *     conditionalForwarderIpAddrs: oneDirectory.dnsIpAddresses,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import the Trust relationship using the directory ID and remote domain name, separated by a `/`. For example:
 *
 * ```sh
 * $ pulumi import aws:directoryservice/trust:Trust example d-926724cf57/directory.example.com
 * ```
 */
export declare class Trust extends pulumi.CustomResource {
    /**
     * Get an existing Trust resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TrustState, opts?: pulumi.CustomResourceOptions): Trust;
    /**
     * Returns true if the given object is an instance of Trust.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Trust;
    /**
     * Set of IPv4 addresses for the DNS server associated with the remote Directory.
     * Can contain between 1 and 4 values.
     */
    readonly conditionalForwarderIpAddrs: pulumi.Output<string[] | undefined>;
    /**
     * Date and time when the Trust was created.
     */
    readonly createdDateTime: pulumi.Output<string>;
    /**
     * Whether to delete the conditional forwarder when deleting the Trust relationship.
     */
    readonly deleteAssociatedConditionalForwarder: pulumi.Output<boolean>;
    /**
     * ID of the Directory.
     */
    readonly directoryId: pulumi.Output<string>;
    /**
     * Date and time when the Trust was last updated.
     */
    readonly lastUpdatedDateTime: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Fully qualified domain name of the remote Directory.
     */
    readonly remoteDomainName: pulumi.Output<string>;
    /**
     * Whether to enable selective authentication.
     * Valid values are `Enabled` and `Disabled`.
     * Default value is `Disabled`.
     */
    readonly selectiveAuth: pulumi.Output<string>;
    /**
     * Date and time when the Trust state in `trustState` was last updated.
     */
    readonly stateLastUpdatedDateTime: pulumi.Output<string>;
    /**
     * The direction of the Trust relationship.
     * Valid values are `One-Way: Outgoing`, `One-Way: Incoming`, and `Two-Way`.
     */
    readonly trustDirection: pulumi.Output<string>;
    /**
     * Password for the Trust.
     * Does not need to match the passwords for either Directory.
     * Can contain upper- and lower-case letters, numbers, and punctuation characters.
     * May be up to 128 characters long.
     */
    readonly trustPassword: pulumi.Output<string>;
    /**
     * State of the Trust relationship.
     * One of `Created`, `VerifyFailed`,`Verified`, `UpdateFailed`,`Updated`,`Deleted`, or `Failed`.
     */
    readonly trustState: pulumi.Output<string>;
    /**
     * Reason for the Trust state set in `trustState`.
     */
    readonly trustStateReason: pulumi.Output<string>;
    /**
     * Type of the Trust relationship.
     * Valid values are `Forest` and `External`.
     * Default value is `Forest`.
     */
    readonly trustType: pulumi.Output<string>;
    /**
     * Create a Trust resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: TrustArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Trust resources.
 */
export interface TrustState {
    /**
     * Set of IPv4 addresses for the DNS server associated with the remote Directory.
     * Can contain between 1 and 4 values.
     */
    conditionalForwarderIpAddrs?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Date and time when the Trust was created.
     */
    createdDateTime?: pulumi.Input<string>;
    /**
     * Whether to delete the conditional forwarder when deleting the Trust relationship.
     */
    deleteAssociatedConditionalForwarder?: pulumi.Input<boolean>;
    /**
     * ID of the Directory.
     */
    directoryId?: pulumi.Input<string>;
    /**
     * Date and time when the Trust was last updated.
     */
    lastUpdatedDateTime?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Fully qualified domain name of the remote Directory.
     */
    remoteDomainName?: pulumi.Input<string>;
    /**
     * Whether to enable selective authentication.
     * Valid values are `Enabled` and `Disabled`.
     * Default value is `Disabled`.
     */
    selectiveAuth?: pulumi.Input<string>;
    /**
     * Date and time when the Trust state in `trustState` was last updated.
     */
    stateLastUpdatedDateTime?: pulumi.Input<string>;
    /**
     * The direction of the Trust relationship.
     * Valid values are `One-Way: Outgoing`, `One-Way: Incoming`, and `Two-Way`.
     */
    trustDirection?: pulumi.Input<string>;
    /**
     * Password for the Trust.
     * Does not need to match the passwords for either Directory.
     * Can contain upper- and lower-case letters, numbers, and punctuation characters.
     * May be up to 128 characters long.
     */
    trustPassword?: pulumi.Input<string>;
    /**
     * State of the Trust relationship.
     * One of `Created`, `VerifyFailed`,`Verified`, `UpdateFailed`,`Updated`,`Deleted`, or `Failed`.
     */
    trustState?: pulumi.Input<string>;
    /**
     * Reason for the Trust state set in `trustState`.
     */
    trustStateReason?: pulumi.Input<string>;
    /**
     * Type of the Trust relationship.
     * Valid values are `Forest` and `External`.
     * Default value is `Forest`.
     */
    trustType?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Trust resource.
 */
export interface TrustArgs {
    /**
     * Set of IPv4 addresses for the DNS server associated with the remote Directory.
     * Can contain between 1 and 4 values.
     */
    conditionalForwarderIpAddrs?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Whether to delete the conditional forwarder when deleting the Trust relationship.
     */
    deleteAssociatedConditionalForwarder?: pulumi.Input<boolean>;
    /**
     * ID of the Directory.
     */
    directoryId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Fully qualified domain name of the remote Directory.
     */
    remoteDomainName: pulumi.Input<string>;
    /**
     * Whether to enable selective authentication.
     * Valid values are `Enabled` and `Disabled`.
     * Default value is `Disabled`.
     */
    selectiveAuth?: pulumi.Input<string>;
    /**
     * The direction of the Trust relationship.
     * Valid values are `One-Way: Outgoing`, `One-Way: Incoming`, and `Two-Way`.
     */
    trustDirection: pulumi.Input<string>;
    /**
     * Password for the Trust.
     * Does not need to match the passwords for either Directory.
     * Can contain upper- and lower-case letters, numbers, and punctuation characters.
     * May be up to 128 characters long.
     */
    trustPassword: pulumi.Input<string>;
    /**
     * Type of the Trust relationship.
     * Valid values are `Forest` and `External`.
     * Default value is `Forest`.
     */
    trustType?: pulumi.Input<string>;
}
