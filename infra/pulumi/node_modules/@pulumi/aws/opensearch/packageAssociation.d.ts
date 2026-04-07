import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an AWS Opensearch Package Association.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const myDomain = new aws.opensearch.Domain("my_domain", {
 *     domainName: "my-opensearch-domain",
 *     engineVersion: "Elasticsearch_7.10",
 *     clusterConfig: {
 *         instanceType: "r4.large.search",
 *     },
 * });
 * const example = new aws.opensearch.Package("example", {
 *     packageName: "example-txt",
 *     packageSource: {
 *         s3BucketName: myOpensearchPackages.bucket,
 *         s3Key: exampleAwsS3Object.key,
 *     },
 *     packageType: "TXT-DICTIONARY",
 * });
 * const examplePackageAssociation = new aws.opensearch.PackageAssociation("example", {
 *     packageId: example.id,
 *     domainName: myDomain.domainName,
 * });
 * ```
 */
export declare class PackageAssociation extends pulumi.CustomResource {
    /**
     * Get an existing PackageAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: PackageAssociationState, opts?: pulumi.CustomResourceOptions): PackageAssociation;
    /**
     * Returns true if the given object is an instance of PackageAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is PackageAssociation;
    /**
     * Name of the domain to associate the package with.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * Internal ID of the package to associate with a domain.
     */
    readonly packageId: pulumi.Output<string>;
    readonly referencePath: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a PackageAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: PackageAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering PackageAssociation resources.
 */
export interface PackageAssociationState {
    /**
     * Name of the domain to associate the package with.
     */
    domainName?: pulumi.Input<string>;
    /**
     * Internal ID of the package to associate with a domain.
     */
    packageId?: pulumi.Input<string>;
    referencePath?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a PackageAssociation resource.
 */
export interface PackageAssociationArgs {
    /**
     * Name of the domain to associate the package with.
     */
    domainName: pulumi.Input<string>;
    /**
     * Internal ID of the package to associate with a domain.
     */
    packageId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
