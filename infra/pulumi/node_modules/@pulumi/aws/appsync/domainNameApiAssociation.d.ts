import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AppSync API Association.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appsync.DomainNameApiAssociation("example", {
 *     apiId: exampleAwsAppsyncGraphqlApi.id,
 *     domainName: exampleAwsAppsyncDomainName.domainName,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_appsync_domain_name_api_association` using the AppSync domain name. For example:
 *
 * ```sh
 * $ pulumi import aws:appsync/domainNameApiAssociation:DomainNameApiAssociation example example.com
 * ```
 */
export declare class DomainNameApiAssociation extends pulumi.CustomResource {
    /**
     * Get an existing DomainNameApiAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DomainNameApiAssociationState, opts?: pulumi.CustomResourceOptions): DomainNameApiAssociation;
    /**
     * Returns true if the given object is an instance of DomainNameApiAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DomainNameApiAssociation;
    /**
     * API ID.
     */
    readonly apiId: pulumi.Output<string>;
    /**
     * Appsync domain name.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a DomainNameApiAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DomainNameApiAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DomainNameApiAssociation resources.
 */
export interface DomainNameApiAssociationState {
    /**
     * API ID.
     */
    apiId?: pulumi.Input<string>;
    /**
     * Appsync domain name.
     */
    domainName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DomainNameApiAssociation resource.
 */
export interface DomainNameApiAssociationArgs {
    /**
     * API ID.
     */
    apiId: pulumi.Input<string>;
    /**
     * Appsync domain name.
     */
    domainName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
