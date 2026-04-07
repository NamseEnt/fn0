import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS Redshift Serverless Custom Domain Association.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.acm.Certificate("example", {domainName: "example.com"});
 * const exampleNamespace = new aws.redshiftserverless.Namespace("example", {namespaceName: "example-namespace"});
 * const exampleWorkgroup = new aws.redshiftserverless.Workgroup("example", {
 *     workgroupName: "example-workgroup",
 *     namespaceName: exampleNamespace.namespaceName,
 * });
 * const exampleCustomDomainAssociation = new aws.redshiftserverless.CustomDomainAssociation("example", {
 *     workgroupName: exampleWorkgroup.workgroupName,
 *     customDomainName: "example.com",
 *     customDomainCertificateArn: example.arn,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift Serverless Custom Domain Association using the `workgroup_name` and `custom_domain_name`, separated by the coma. For example:
 *
 * ```sh
 * $ pulumi import aws:redshiftserverless/customDomainAssociation:CustomDomainAssociation example example-workgroup,example.com
 * ```
 */
export declare class CustomDomainAssociation extends pulumi.CustomResource {
    /**
     * Get an existing CustomDomainAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CustomDomainAssociationState, opts?: pulumi.CustomResourceOptions): CustomDomainAssociation;
    /**
     * Returns true if the given object is an instance of CustomDomainAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is CustomDomainAssociation;
    /**
     * ARN of the certificate for the custom domain association.
     */
    readonly customDomainCertificateArn: pulumi.Output<string>;
    /**
     * Expiration time for the certificate.
     */
    readonly customDomainCertificateExpiryTime: pulumi.Output<string>;
    /**
     * Custom domain to associate with the workgroup.
     */
    readonly customDomainName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name of the workgroup.
     */
    readonly workgroupName: pulumi.Output<string>;
    /**
     * Create a CustomDomainAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: CustomDomainAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering CustomDomainAssociation resources.
 */
export interface CustomDomainAssociationState {
    /**
     * ARN of the certificate for the custom domain association.
     */
    customDomainCertificateArn?: pulumi.Input<string>;
    /**
     * Expiration time for the certificate.
     */
    customDomainCertificateExpiryTime?: pulumi.Input<string>;
    /**
     * Custom domain to associate with the workgroup.
     */
    customDomainName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the workgroup.
     */
    workgroupName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a CustomDomainAssociation resource.
 */
export interface CustomDomainAssociationArgs {
    /**
     * ARN of the certificate for the custom domain association.
     */
    customDomainCertificateArn: pulumi.Input<string>;
    /**
     * Custom domain to associate with the workgroup.
     */
    customDomainName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the workgroup.
     */
    workgroupName: pulumi.Input<string>;
}
