import * as pulumi from "@pulumi/pulumi";
/**
 * Manages a Lightsail domain entry (DNS record). Use this resource to define how DNS queries for your domain are handled.
 *
 * > **NOTE on `id`:** In an effort to simplify imports, this resource `id` field has been updated to the standard resource id separator, a comma (`,`). For backward compatibility, the previous separator (underscore `_`) can still be used to read and import existing resources. When state is refreshed, the `id` will be updated to use the new standard separator. The previous separator will be deprecated in a future major release.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lightsail.Domain("example", {domainName: "example.com"});
 * const exampleDomainEntry = new aws.lightsail.DomainEntry("example", {
 *     domainName: example.domainName,
 *     name: "www",
 *     type: "A",
 *     target: "127.0.0.1",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Lightsail Domain Entry using the id attribute. For example:
 *
 * ```sh
 * $ pulumi import aws:lightsail/domainEntry:DomainEntry example www,example.com,A,127.0.0.1
 * ```
 */
export declare class DomainEntry extends pulumi.CustomResource {
    /**
     * Get an existing DomainEntry resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DomainEntryState, opts?: pulumi.CustomResourceOptions): DomainEntry;
    /**
     * Returns true if the given object is an instance of DomainEntry.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DomainEntry;
    /**
     * Name of the Lightsail domain in which to create the entry.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * Whether the entry should be an alias. Default: `false`.
     */
    readonly isAlias: pulumi.Output<boolean | undefined>;
    /**
     * Name of the entry record.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Target of the domain entry.
     */
    readonly target: pulumi.Output<string>;
    /**
     * Type of record. Valid values: `A`, `AAAA`, `CNAME`, `MX`, `NS`, `SOA`, `SRV`, `TXT`.
     *
     * The following arguments are optional:
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a DomainEntry resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DomainEntryArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DomainEntry resources.
 */
export interface DomainEntryState {
    /**
     * Name of the Lightsail domain in which to create the entry.
     */
    domainName?: pulumi.Input<string>;
    /**
     * Whether the entry should be an alias. Default: `false`.
     */
    isAlias?: pulumi.Input<boolean>;
    /**
     * Name of the entry record.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Target of the domain entry.
     */
    target?: pulumi.Input<string>;
    /**
     * Type of record. Valid values: `A`, `AAAA`, `CNAME`, `MX`, `NS`, `SOA`, `SRV`, `TXT`.
     *
     * The following arguments are optional:
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DomainEntry resource.
 */
export interface DomainEntryArgs {
    /**
     * Name of the Lightsail domain in which to create the entry.
     */
    domainName: pulumi.Input<string>;
    /**
     * Whether the entry should be an alias. Default: `false`.
     */
    isAlias?: pulumi.Input<boolean>;
    /**
     * Name of the entry record.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Target of the domain entry.
     */
    target: pulumi.Input<string>;
    /**
     * Type of record. Valid values: `A`, `AAAA`, `CNAME`, `MX`, `NS`, `SOA`, `SRV`, `TXT`.
     *
     * The following arguments are optional:
     */
    type: pulumi.Input<string>;
}
