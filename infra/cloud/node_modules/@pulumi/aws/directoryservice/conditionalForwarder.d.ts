import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a conditional forwarder for managed Microsoft AD in AWS Directory Service.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.directoryservice.ConditionalForwarder("example", {
 *     directoryId: ad.id,
 *     remoteDomainName: "example.com",
 *     dnsIps: [
 *         "8.8.8.8",
 *         "8.8.4.4",
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import conditional forwarders using the directory id and remote_domain_name. For example:
 *
 * ```sh
 * $ pulumi import aws:directoryservice/conditionalForwarder:ConditionalForwarder example d-1234567890:example.com
 * ```
 */
export declare class ConditionalForwarder extends pulumi.CustomResource {
    /**
     * Get an existing ConditionalForwarder resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConditionalForwarderState, opts?: pulumi.CustomResourceOptions): ConditionalForwarder;
    /**
     * Returns true if the given object is an instance of ConditionalForwarder.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ConditionalForwarder;
    /**
     * ID of directory.
     */
    readonly directoryId: pulumi.Output<string>;
    /**
     * A list of forwarder IP addresses.
     */
    readonly dnsIps: pulumi.Output<string[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The fully qualified domain name of the remote domain for which forwarders will be used.
     */
    readonly remoteDomainName: pulumi.Output<string>;
    /**
     * Create a ConditionalForwarder resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ConditionalForwarderArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ConditionalForwarder resources.
 */
export interface ConditionalForwarderState {
    /**
     * ID of directory.
     */
    directoryId?: pulumi.Input<string>;
    /**
     * A list of forwarder IP addresses.
     */
    dnsIps?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The fully qualified domain name of the remote domain for which forwarders will be used.
     */
    remoteDomainName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ConditionalForwarder resource.
 */
export interface ConditionalForwarderArgs {
    /**
     * ID of directory.
     */
    directoryId: pulumi.Input<string>;
    /**
     * A list of forwarder IP addresses.
     */
    dnsIps: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The fully qualified domain name of the remote domain for which forwarders will be used.
     */
    remoteDomainName: pulumi.Input<string>;
}
