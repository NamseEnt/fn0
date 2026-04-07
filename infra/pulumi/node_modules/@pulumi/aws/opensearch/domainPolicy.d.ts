import * as pulumi from "@pulumi/pulumi";
/**
 * Allows setting policy to an OpenSearch domain while referencing domain attributes (e.g., ARN).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.opensearch.Domain("example", {
 *     domainName: "tf-test",
 *     engineVersion: "OpenSearch_1.1",
 * });
 * const main = aws.iam.getPolicyDocumentOutput({
 *     statements: [{
 *         effect: "Allow",
 *         principals: [{
 *             type: "*",
 *             identifiers: ["*"],
 *         }],
 *         actions: ["es:*"],
 *         resources: [pulumi.interpolate`${example.arn}/*`],
 *         conditions: [{
 *             test: "IpAddress",
 *             variable: "aws:SourceIp",
 *             values: ["127.0.0.1/32"],
 *         }],
 *     }],
 * });
 * const mainDomainPolicy = new aws.opensearch.DomainPolicy("main", {
 *     domainName: example.domainName,
 *     accessPolicies: main.apply(main => main.json),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import OpenSearch Domain Policy using `domain_name` prefixed with `esd-policy-`. For example:
 *
 * ```sh
 * $ pulumi import aws:opensearch/domainPolicy:DomainPolicy example esd-policy-tf-test
 * ```
 */
export declare class DomainPolicy extends pulumi.CustomResource {
    /**
     * Get an existing DomainPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DomainPolicyState, opts?: pulumi.CustomResourceOptions): DomainPolicy;
    /**
     * Returns true if the given object is an instance of DomainPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DomainPolicy;
    /**
     * IAM policy document specifying the access policies for the domain
     */
    readonly accessPolicies: pulumi.Output<string>;
    /**
     * Name of the domain.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a DomainPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DomainPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DomainPolicy resources.
 */
export interface DomainPolicyState {
    /**
     * IAM policy document specifying the access policies for the domain
     */
    accessPolicies?: pulumi.Input<string>;
    /**
     * Name of the domain.
     */
    domainName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DomainPolicy resource.
 */
export interface DomainPolicyArgs {
    /**
     * IAM policy document specifying the access policies for the domain
     */
    accessPolicies: pulumi.Input<string>;
    /**
     * Name of the domain.
     */
    domainName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
