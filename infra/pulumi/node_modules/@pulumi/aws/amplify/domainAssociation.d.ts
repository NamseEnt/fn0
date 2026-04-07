import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an Amplify Domain Association resource.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.amplify.App("example", {
 *     name: "app",
 *     customRules: [{
 *         source: "https://example.com",
 *         status: "302",
 *         target: "https://www.example.com",
 *     }],
 * });
 * const master = new aws.amplify.Branch("master", {
 *     appId: example.id,
 *     branchName: "master",
 * });
 * const exampleDomainAssociation = new aws.amplify.DomainAssociation("example", {
 *     appId: example.id,
 *     domainName: "example.com",
 *     subDomains: [
 *         {
 *             branchName: master.branchName,
 *             prefix: "",
 *         },
 *         {
 *             branchName: master.branchName,
 *             prefix: "www",
 *         },
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Amplify domain association using `app_id` and `domain_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:amplify/domainAssociation:DomainAssociation app d2ypk4k47z8u6/example.com
 * ```
 */
export declare class DomainAssociation extends pulumi.CustomResource {
    /**
     * Get an existing DomainAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DomainAssociationState, opts?: pulumi.CustomResourceOptions): DomainAssociation;
    /**
     * Returns true if the given object is an instance of DomainAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DomainAssociation;
    /**
     * Unique ID for an Amplify app.
     */
    readonly appId: pulumi.Output<string>;
    /**
     * ARN for the domain association.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The type of SSL/TLS certificate to use for your custom domain. If you don't specify a certificate type, Amplify uses the default certificate that it provisions and manages for you.
     */
    readonly certificateSettings: pulumi.Output<outputs.amplify.DomainAssociationCertificateSettings>;
    /**
     * DNS records for certificate verification in a space-delimited format (`<record> CNAME <target>`).
     */
    readonly certificateVerificationDnsRecord: pulumi.Output<string>;
    /**
     * Domain name for the domain association.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * Enables the automated creation of subdomains for branches.
     */
    readonly enableAutoSubDomain: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Setting for the subdomain. Documented below.
     */
    readonly subDomains: pulumi.Output<outputs.amplify.DomainAssociationSubDomain[]>;
    /**
     * If enabled, the resource will wait for the domain association status to change to `PENDING_DEPLOYMENT` or `AVAILABLE`. Setting this to `false` will skip the process. Default: `true`.
     */
    readonly waitForVerification: pulumi.Output<boolean | undefined>;
    /**
     * Create a DomainAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DomainAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DomainAssociation resources.
 */
export interface DomainAssociationState {
    /**
     * Unique ID for an Amplify app.
     */
    appId?: pulumi.Input<string>;
    /**
     * ARN for the domain association.
     */
    arn?: pulumi.Input<string>;
    /**
     * The type of SSL/TLS certificate to use for your custom domain. If you don't specify a certificate type, Amplify uses the default certificate that it provisions and manages for you.
     */
    certificateSettings?: pulumi.Input<inputs.amplify.DomainAssociationCertificateSettings>;
    /**
     * DNS records for certificate verification in a space-delimited format (`<record> CNAME <target>`).
     */
    certificateVerificationDnsRecord?: pulumi.Input<string>;
    /**
     * Domain name for the domain association.
     */
    domainName?: pulumi.Input<string>;
    /**
     * Enables the automated creation of subdomains for branches.
     */
    enableAutoSubDomain?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Setting for the subdomain. Documented below.
     */
    subDomains?: pulumi.Input<pulumi.Input<inputs.amplify.DomainAssociationSubDomain>[]>;
    /**
     * If enabled, the resource will wait for the domain association status to change to `PENDING_DEPLOYMENT` or `AVAILABLE`. Setting this to `false` will skip the process. Default: `true`.
     */
    waitForVerification?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a DomainAssociation resource.
 */
export interface DomainAssociationArgs {
    /**
     * Unique ID for an Amplify app.
     */
    appId: pulumi.Input<string>;
    /**
     * The type of SSL/TLS certificate to use for your custom domain. If you don't specify a certificate type, Amplify uses the default certificate that it provisions and manages for you.
     */
    certificateSettings?: pulumi.Input<inputs.amplify.DomainAssociationCertificateSettings>;
    /**
     * Domain name for the domain association.
     */
    domainName: pulumi.Input<string>;
    /**
     * Enables the automated creation of subdomains for branches.
     */
    enableAutoSubDomain?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Setting for the subdomain. Documented below.
     */
    subDomains: pulumi.Input<pulumi.Input<inputs.amplify.DomainAssociationSubDomain>[]>;
    /**
     * If enabled, the resource will wait for the domain association status to change to `PENDING_DEPLOYMENT` or `AVAILABLE`. Setting this to `false` will skip the process. Default: `true`.
     */
    waitForVerification?: pulumi.Input<boolean>;
}
