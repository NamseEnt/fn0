import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.route53domains.Domain("example", {
 *     domainName: "example.com",
 *     autoRenew: false,
 *     adminContact: {
 *         addressLine1: "101 Main Street",
 *         city: "San Francisco",
 *         contactType: "COMPANY",
 *         countryCode: "US",
 *         email: "pulumi-acctest@example.com",
 *         fax: "+1.4155551234",
 *         firstName: "Terraform",
 *         lastName: "Team",
 *         organizationName: "HashiCorp",
 *         phoneNumber: "+1.4155551234",
 *         state: "CA",
 *         zipCode: "94105",
 *     },
 *     registrantContact: {
 *         addressLine1: "101 Main Street",
 *         city: "San Francisco",
 *         contactType: "COMPANY",
 *         countryCode: "US",
 *         email: "pulumi-acctest@example.com",
 *         fax: "+1.4155551234",
 *         firstName: "Terraform",
 *         lastName: "Team",
 *         organizationName: "HashiCorp",
 *         phoneNumber: "+1.4155551234",
 *         state: "CA",
 *         zipCode: "94105",
 *     },
 *     techContact: {
 *         addressLine1: "101 Main Street",
 *         city: "San Francisco",
 *         contactType: "COMPANY",
 *         countryCode: "US",
 *         email: "pulumi-acctest@example.com",
 *         fax: "+1.4155551234",
 *         firstName: "Terraform",
 *         lastName: "Team",
 *         organizationName: "HashiCorp",
 *         phoneNumber: "+1.4155551234",
 *         state: "CA",
 *         zipCode: "94105",
 *     },
 *     tags: {
 *         Environment: "test",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import domains using the `domain_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:route53domains/domain:Domain example example.com
 * ```
 */
export declare class Domain extends pulumi.CustomResource {
    /**
     * Get an existing Domain resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DomainState, opts?: pulumi.CustomResourceOptions): Domain;
    /**
     * Returns true if the given object is an instance of Domain.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Domain;
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
    readonly adminContact: pulumi.Output<outputs.route53domains.DomainAdminContact | undefined>;
    /**
     * Whether domain administrative contact information is concealed from WHOIS queries. Default: `true`.
     */
    readonly adminPrivacy: pulumi.Output<boolean>;
    /**
     * Whether the domain registration is set to renew automatically. Default: `true`.
     */
    readonly autoRenew: pulumi.Output<boolean>;
    /**
     * Details about the domain billing contact. See Contact Blocks for more details.
     */
    readonly billingContacts: pulumi.Output<outputs.route53domains.DomainBillingContact[]>;
    /**
     * Whether domain billing contact information is concealed from WHOIS queries. Default: `true`.
     */
    readonly billingPrivacy: pulumi.Output<boolean>;
    /**
     * The date when the domain was created as found in the response to a WHOIS query.
     */
    readonly creationDate: pulumi.Output<string>;
    /**
     * The name of the domain.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * The number of years that you want to register the domain for. Domains are registered for a minimum of one year. Increasing the duration renews the domain.
     */
    readonly durationInYears: pulumi.Output<number>;
    /**
     * The date when the registration for the domain is set to expire.
     */
    readonly expirationDate: pulumi.Output<string>;
    /**
     * The ID of the public Route 53 hosted zone created for the domain. This hosted zone is deleted when the domain is deregistered.
     */
    readonly hostedZoneId: pulumi.Output<string>;
    /**
     * The list of nameservers for the domain. See `nameServer` Blocks for more details.
     */
    readonly nameServers: pulumi.Output<outputs.route53domains.DomainNameServer[]>;
    /**
     * Details about the domain registrant. See Contact Blocks for more details.
     */
    readonly registrantContact: pulumi.Output<outputs.route53domains.DomainRegistrantContact | undefined>;
    /**
     * Whether domain registrant contact information is concealed from WHOIS queries. Default: `true`.
     */
    readonly registrantPrivacy: pulumi.Output<boolean>;
    /**
     * Name of the registrar of the domain as identified in the registry.
     */
    readonly registrarName: pulumi.Output<string>;
    /**
     * Web address of the registrar.
     */
    readonly registrarUrl: pulumi.Output<string>;
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
    readonly techContact: pulumi.Output<outputs.route53domains.DomainTechContact | undefined>;
    /**
     * Whether domain technical contact information is concealed from WHOIS queries. Default: `true`.
     */
    readonly techPrivacy: pulumi.Output<boolean>;
    readonly timeouts: pulumi.Output<outputs.route53domains.DomainTimeouts | undefined>;
    /**
     * Whether the domain is locked for transfer. Default: `true`.
     *
     * > **NOTE:** You must specify the same privacy setting for `adminPrivacy`, `registrantPrivacy` and `techPrivacy`.
     */
    readonly transferLock: pulumi.Output<boolean>;
    /**
     * The last updated date of the domain as found in the response to a WHOIS query.
     */
    readonly updatedDate: pulumi.Output<string>;
    /**
     * The fully qualified name of the WHOIS server that can answer the WHOIS query for the domain.
     */
    readonly whoisServer: pulumi.Output<string>;
    /**
     * Create a Domain resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DomainArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Domain resources.
 */
export interface DomainState {
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
    adminContact?: pulumi.Input<inputs.route53domains.DomainAdminContact>;
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
    billingContacts?: pulumi.Input<pulumi.Input<inputs.route53domains.DomainBillingContact>[]>;
    /**
     * Whether domain billing contact information is concealed from WHOIS queries. Default: `true`.
     */
    billingPrivacy?: pulumi.Input<boolean>;
    /**
     * The date when the domain was created as found in the response to a WHOIS query.
     */
    creationDate?: pulumi.Input<string>;
    /**
     * The name of the domain.
     */
    domainName?: pulumi.Input<string>;
    /**
     * The number of years that you want to register the domain for. Domains are registered for a minimum of one year. Increasing the duration renews the domain.
     */
    durationInYears?: pulumi.Input<number>;
    /**
     * The date when the registration for the domain is set to expire.
     */
    expirationDate?: pulumi.Input<string>;
    /**
     * The ID of the public Route 53 hosted zone created for the domain. This hosted zone is deleted when the domain is deregistered.
     */
    hostedZoneId?: pulumi.Input<string>;
    /**
     * The list of nameservers for the domain. See `nameServer` Blocks for more details.
     */
    nameServers?: pulumi.Input<pulumi.Input<inputs.route53domains.DomainNameServer>[]>;
    /**
     * Details about the domain registrant. See Contact Blocks for more details.
     */
    registrantContact?: pulumi.Input<inputs.route53domains.DomainRegistrantContact>;
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
    techContact?: pulumi.Input<inputs.route53domains.DomainTechContact>;
    /**
     * Whether domain technical contact information is concealed from WHOIS queries. Default: `true`.
     */
    techPrivacy?: pulumi.Input<boolean>;
    timeouts?: pulumi.Input<inputs.route53domains.DomainTimeouts>;
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
 * The set of arguments for constructing a Domain resource.
 */
export interface DomainArgs {
    /**
     * Details about the domain administrative contact. See Contact Blocks for more details.
     */
    adminContact?: pulumi.Input<inputs.route53domains.DomainAdminContact>;
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
    billingContacts?: pulumi.Input<pulumi.Input<inputs.route53domains.DomainBillingContact>[]>;
    /**
     * Whether domain billing contact information is concealed from WHOIS queries. Default: `true`.
     */
    billingPrivacy?: pulumi.Input<boolean>;
    /**
     * The name of the domain.
     */
    domainName: pulumi.Input<string>;
    /**
     * The number of years that you want to register the domain for. Domains are registered for a minimum of one year. Increasing the duration renews the domain.
     */
    durationInYears?: pulumi.Input<number>;
    /**
     * The list of nameservers for the domain. See `nameServer` Blocks for more details.
     */
    nameServers?: pulumi.Input<pulumi.Input<inputs.route53domains.DomainNameServer>[]>;
    /**
     * Details about the domain registrant. See Contact Blocks for more details.
     */
    registrantContact?: pulumi.Input<inputs.route53domains.DomainRegistrantContact>;
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
    techContact?: pulumi.Input<inputs.route53domains.DomainTechContact>;
    /**
     * Whether domain technical contact information is concealed from WHOIS queries. Default: `true`.
     */
    techPrivacy?: pulumi.Input<boolean>;
    timeouts?: pulumi.Input<inputs.route53domains.DomainTimeouts>;
    /**
     * Whether the domain is locked for transfer. Default: `true`.
     *
     * > **NOTE:** You must specify the same privacy setting for `adminPrivacy`, `registrantPrivacy` and `techPrivacy`.
     */
    transferLock?: pulumi.Input<boolean>;
}
