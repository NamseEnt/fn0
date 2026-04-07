import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an AppSync Domain Name.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.appsync.DomainName("example", {
 *     domainName: "api.example.com",
 *     certificateArn: exampleAwsAcmCertificate.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_appsync_domain_name` using the AppSync domain name. For example:
 *
 * ```sh
 * $ pulumi import aws:appsync/domainName:DomainName example example.com
 * ```
 */
export declare class DomainName extends pulumi.CustomResource {
    /**
     * Get an existing DomainName resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DomainNameState, opts?: pulumi.CustomResourceOptions): DomainName;
    /**
     * Returns true if the given object is an instance of DomainName.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DomainName;
    /**
     * Domain name that AppSync provides.
     */
    readonly appsyncDomainName: pulumi.Output<string>;
    /**
     * ARN of the certificate. This can be an Certificate Manager (ACM) certificate or an Identity and Access Management (IAM) server certificate. The certifiacte must reside in us-east-1.
     */
    readonly certificateArn: pulumi.Output<string>;
    /**
     * A description of the Domain Name.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Domain name.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * ID of your Amazon Route 53 hosted zone.
     */
    readonly hostedZoneId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a DomainName resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DomainNameArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DomainName resources.
 */
export interface DomainNameState {
    /**
     * Domain name that AppSync provides.
     */
    appsyncDomainName?: pulumi.Input<string>;
    /**
     * ARN of the certificate. This can be an Certificate Manager (ACM) certificate or an Identity and Access Management (IAM) server certificate. The certifiacte must reside in us-east-1.
     */
    certificateArn?: pulumi.Input<string>;
    /**
     * A description of the Domain Name.
     */
    description?: pulumi.Input<string>;
    /**
     * Domain name.
     */
    domainName?: pulumi.Input<string>;
    /**
     * ID of your Amazon Route 53 hosted zone.
     */
    hostedZoneId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DomainName resource.
 */
export interface DomainNameArgs {
    /**
     * ARN of the certificate. This can be an Certificate Manager (ACM) certificate or an Identity and Access Management (IAM) server certificate. The certifiacte must reside in us-east-1.
     */
    certificateArn: pulumi.Input<string>;
    /**
     * A description of the Domain Name.
     */
    description?: pulumi.Input<string>;
    /**
     * Domain name.
     */
    domainName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
