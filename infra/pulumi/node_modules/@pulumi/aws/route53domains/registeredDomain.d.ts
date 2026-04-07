import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage a domain that has been [registered](https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/registrar-tld-list.html) and associated with the current AWS account. To register, renew and deregister a domain use the `aws.route53domains.Domain` resource instead.
 *
 * **This is an advanced resource** and has special caveats to be aware of when using it. Please read this document in its entirety before using this resource.
 *
 * The `aws.route53domains.RegisteredDomain` resource behaves differently from normal resources in that if a domain has been registered, the provider does not _register_ this domain, but instead "adopts" it into management. A destroy does not delete the domain but does remove the resource from state.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.route53domains.RegisteredDomain("example", {
 *     domainName: "example.com",
 *     nameServers: [
 *         {
 *             name: "ns-195.awsdns-24.com",
 *         },
 *         {
 *             name: "ns-874.awsdns-45.net",
 *         },
 *     ],
 *     tags: {
 *         Environment: "test",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import domains using the domain name. For example:
 *
 * ```sh
 * $ pulumi import aws:route53domains/registeredDomain:RegisteredDomain example example.com
 * ```
 */
export declare class RegisteredDomain extends pulumi.CustomResource {
    /**
     * Get an existing RegisteredDomain resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: RegisteredDomainState, opts?: pulumi.CustomResourceOptions): RegisteredDomain;
    /**
     * Returns true if the given object is an instance of RegisteredDomain.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is RegisteredDomain;
    /**
     * Email address to contact to report incorrect contact information for a domain, to report that the domain is being used to send spam, to report that someone is cybersquatting on a domain name, or report some other type of abuse.
     */
    readonly abuseContactEmail: pulumi.Output<string>;
    /**
     * Phone number for reporting abuse.
     */
    readonly abuseContactPhone: pulumi.Output<string>;
    /**
     * Details about the domain administrative contact. See Contact Blocks for more details.
     */
    readonly adminContact: pulumi.Output<outputs.route53domains.RegisteredDomainAdminContact>;
    /**
     * Whether domain administrative contact information is concealed from WHOIS queries. Default: `true`.
     */
    readonly adminPrivacy: pulumi.Output<boolean | undefined>;
    /**
     * Whether the domain registration is set to renew automatically. Default: `true`.
     */
    readonly autoRenew: pulumi.Output<boolean | undefined>;
    /**
     * Details about the domain billing contact. See Contact Blocks for more details.
     */
    readonly billingContact: pulumi.Output<outputs.route53domains.RegisteredDomainBillingContact>;
    /**
     * Whether domain billing contact information is concealed from WHOIS queries. Default: `true`.
     */
    readonly billingPrivacy: pulumi.Output<boolean | undefined>;
    /**
     * The date when the domain was created as found in the response to a WHOIS query.
     */
    readonly creationDate: pulumi.Output<string>;
    /**
     * The name of the registered domain.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * The date when the registration for the domain is set to expire.
     */
    readonly expirationDate: pulumi.Output<string>;
    /**
     * The list of nameservers for the domain. See `nameServer` Blocks for more details.
     */
    readonly nameServers: pulumi.Output<outputs.route53domains.RegisteredDomainNameServer[]>;
    /**
     * Details about the domain registrant. See Contact Blocks for more details.
     */
    readonly registrantContact: pulumi.Output<outputs.route53domains.RegisteredDomainRegistrantContact>;
    /**
     * Whether domain registrant contact information is concealed from WHOIS queries. Default: `true`.
     */
    readonly registrantPrivacy: pulumi.Output<boolean | undefined>;
    /**
     * Name of the registrar of the domain as identified in the registry.
     */
    readonly registrarName: pulumi.Output<string>;
    /**
     * Web address of the registrar.
     */
    readonly registrarUrl: pulumi.Output<string>;
    /**
     * Reseller of the domain.
     */
    readonly reseller: pulumi.Output<string>;
    /**
     * List of [domain name status codes](https://www.icann.org/resources/pages/epp-status-codes-2014-06-16-en).
     */
    readonly statusLists: pulumi.Output<string[]>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Details about the domain technical contact. See Contact Blocks for more details.
     */
    readonly techContact: pulumi.Output<outputs.route53domains.RegisteredDomainTechContact>;
    /**
     * Whether domain technical contact information is concealed from WHOIS queries. Default: `true`.
     */
    readonly techPrivacy: pulumi.Output<boolean | undefined>;
    /**
     * Whether the domain is locked for transfer. Default: `true`.
     *
     * > **NOTE:** You must specify the same privacy setting for `adminPrivacy`, `registrantPrivacy` and `techPrivacy`.
     */
    readonly transferLock: pulumi.Output<boolean | undefined>;
    /**
     * The last updated date of the domain as found in the response to a WHOIS query.
     */
    readonly updatedDate: pulumi.Output<string>;
    /**
     * The fully qualified name of the WHOIS server that can answer the WHOIS query for the domain.
     */
    readonly whoisServer: pulumi.Output<string>;
    /**
     * Create a RegisteredDomain resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: RegisteredDomainArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering RegisteredDomain resources.
 */
export interface RegisteredDomainState {
    /**
     * Email address to contact to report incorrect contact information for a domain, to report that the domain is being used to send spam, to report that someone is cybersquatting on a domain name, or report some other type of abuse.
     */
    abuseContactEmail?: pulumi.Input<string>;
    /**
     * Phone number for reporting abuse.
     */
    abuseContactPhone?: pulumi.Input<string>;
    /**
     * Details about the domain administrative contact. See Contact Blocks for more details.
     */
    adminContact?: pulumi.Input<inputs.route53domains.RegisteredDomainAdminContact>;
    /**
     * Whether domain administrative contact information is concealed from WHOIS queries. Default: `true`.
     */
    adminPrivacy?: pulumi.Input<boolean>;
    /**
     * Whether the domain registration is set to renew automatically. Default: `true`.
     */
    autoRenew?: pulumi.Input<boolean>;
    /**
     * Details about the domain billing contact. See Contact Blocks for more details.
     */
    billingContact?: pulumi.Input<inputs.route53domains.RegisteredDomainBillingContact>;
    /**
     * Whether domain billing contact information is concealed from WHOIS queries. Default: `true`.
     */
    billingPrivacy?: pulumi.Input<boolean>;
    /**
     * The date when the domain was created as found in the response to a WHOIS query.
     */
    creationDate?: pulumi.Input<string>;
    /**
     * The name of the registered domain.
     */
    domainName?: pulumi.Input<string>;
    /**
     * The date when the registration for the domain is set to expire.
     */
    expirationDate?: pulumi.Input<string>;
    /**
     * The list of nameservers for the domain. See `nameServer` Blocks for more details.
     */
    nameServers?: pulumi.Input<pulumi.Input<inputs.route53domains.RegisteredDomainNameServer>[]>;
    /**
     * Details about the domain registrant. See Contact Blocks for more details.
     */
    registrantContact?: pulumi.Input<inputs.route53domains.RegisteredDomainRegistrantContact>;
    /**
     * Whether domain registrant contact information is concealed from WHOIS queries. Default: `true`.
     */
    registrantPrivacy?: pulumi.Input<boolean>;
    /**
     * Name of the registrar of the domain as identified in the registry.
     */
    registrarName?: pulumi.Input<string>;
    /**
     * Web address of the registrar.
     */
    registrarUrl?: pulumi.Input<string>;
    /**
     * Reseller of the domain.
     */
    reseller?: pulumi.Input<string>;
    /**
     * List of [domain name status codes](https://www.icann.org/resources/pages/epp-status-codes-2014-06-16-en).
     */
    statusLists?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Details about the domain technical contact. See Contact Blocks for more details.
     */
    techContact?: pulumi.Input<inputs.route53domains.RegisteredDomainTechContact>;
    /**
     * Whether domain technical contact information is concealed from WHOIS queries. Default: `true`.
     */
    techPrivacy?: pulumi.Input<boolean>;
    /**
     * Whether the domain is locked for transfer. Default: `true`.
     *
     * > **NOTE:** You must specify the same privacy setting for `adminPrivacy`, `registrantPrivacy` and `techPrivacy`.
     */
    transferLock?: pulumi.Input<boolean>;
    /**
     * The last updated date of the domain as found in the response to a WHOIS query.
     */
    updatedDate?: pulumi.Input<string>;
    /**
     * The fully qualified name of the WHOIS server that can answer the WHOIS query for the domain.
     */
    whoisServer?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a RegisteredDomain resource.
 */
export interface RegisteredDomainArgs {
    /**
     * Details about the domain administrative contact. See Contact Blocks for more details.
     */
    adminContact?: pulumi.Input<inputs.route53domains.RegisteredDomainAdminContact>;
    /**
     * Whether domain administrative contact information is concealed from WHOIS queries. Default: `true`.
     */
    adminPrivacy?: pulumi.Input<boolean>;
    /**
     * Whether the domain registration is set to renew automatically. Default: `true`.
     */
    autoRenew?: pulumi.Input<boolean>;
    /**
     * Details about the domain billing contact. See Contact Blocks for more details.
     */
    billingContact?: pulumi.Input<inputs.route53domains.RegisteredDomainBillingContact>;
    /**
     * Whether domain billing contact information is concealed from WHOIS queries. Default: `true`.
     */
    billingPrivacy?: pulumi.Input<boolean>;
    /**
     * The name of the registered domain.
     */
    domainName: pulumi.Input<string>;
    /**
     * The list of nameservers for the domain. See `nameServer` Blocks for more details.
     */
    nameServers?: pulumi.Input<pulumi.Input<inputs.route53domains.RegisteredDomainNameServer>[]>;
    /**
     * Details about the domain registrant. See Contact Blocks for more details.
     */
    registrantContact?: pulumi.Input<inputs.route53domains.RegisteredDomainRegistrantContact>;
    /**
     * Whether domain registrant contact information is concealed from WHOIS queries. Default: `true`.
     */
    registrantPrivacy?: pulumi.Input<boolean>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Details about the domain technical contact. See Contact Blocks for more details.
     */
    techContact?: pulumi.Input<inputs.route53domains.RegisteredDomainTechContact>;
    /**
     * Whether domain technical contact information is concealed from WHOIS queries. Default: `true`.
     */
    techPrivacy?: pulumi.Input<boolean>;
    /**
     * Whether the domain is locked for transfer. Default: `true`.
     *
     * > **NOTE:** You must specify the same privacy setting for `adminPrivacy`, `registrantPrivacy` and `techPrivacy`.
     */
    transferLock?: pulumi.Input<boolean>;
}
