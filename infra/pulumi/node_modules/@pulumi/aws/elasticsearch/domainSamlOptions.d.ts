import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages SAML authentication options for an AWS Elasticsearch Domain.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = new aws.elasticsearch.Domain("example", {
 *     domainName: "example",
 *     elasticsearchVersion: "1.5",
 *     clusterConfig: {
 *         instanceType: "r4.large.elasticsearch",
 *     },
 *     snapshotOptions: {
 *         automatedSnapshotStartHour: 23,
 *     },
 *     tags: {
 *         Domain: "TestDomain",
 *     },
 * });
 * const exampleDomainSamlOptions = new aws.elasticsearch.DomainSamlOptions("example", {
 *     domainName: example.domainName,
 *     samlOptions: {
 *         enabled: true,
 *         idp: {
 *             entityId: "https://example.com",
 *             metadataContent: std.file({
 *                 input: "./saml-metadata.xml",
 *             }).then(invoke => invoke.result),
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Elasticsearch domains using the `domain_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:elasticsearch/domainSamlOptions:DomainSamlOptions example domain_name
 * ```
 */
export declare class DomainSamlOptions extends pulumi.CustomResource {
    /**
     * Get an existing DomainSamlOptions resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DomainSamlOptionsState, opts?: pulumi.CustomResourceOptions): DomainSamlOptions;
    /**
     * Returns true if the given object is an instance of DomainSamlOptions.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DomainSamlOptions;
    /**
     * Name of the domain.
     *
     * The following arguments are optional:
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The SAML authentication options for an AWS Elasticsearch Domain.
     */
    readonly samlOptions: pulumi.Output<outputs.elasticsearch.DomainSamlOptionsSamlOptions | undefined>;
    /**
     * Create a DomainSamlOptions resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DomainSamlOptionsArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DomainSamlOptions resources.
 */
export interface DomainSamlOptionsState {
    /**
     * Name of the domain.
     *
     * The following arguments are optional:
     */
    domainName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The SAML authentication options for an AWS Elasticsearch Domain.
     */
    samlOptions?: pulumi.Input<inputs.elasticsearch.DomainSamlOptionsSamlOptions>;
}
/**
 * The set of arguments for constructing a DomainSamlOptions resource.
 */
export interface DomainSamlOptionsArgs {
    /**
     * Name of the domain.
     *
     * The following arguments are optional:
     */
    domainName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The SAML authentication options for an AWS Elasticsearch Domain.
     */
    samlOptions?: pulumi.Input<inputs.elasticsearch.DomainSamlOptionsSamlOptions>;
}
