import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an API Gateway Client Certificate.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const demo = new aws.apigateway.ClientCertificate("demo", {description: "My client certificate"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import API Gateway Client Certificates using the id. For example:
 *
 * ```sh
 * $ pulumi import aws:apigateway/clientCertificate:ClientCertificate demo ab1cqe
 * ```
 */
export declare class ClientCertificate extends pulumi.CustomResource {
    /**
     * Get an existing ClientCertificate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ClientCertificateState, opts?: pulumi.CustomResourceOptions): ClientCertificate;
    /**
     * Returns true if the given object is an instance of ClientCertificate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ClientCertificate;
    /**
     * ARN
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Date when the client certificate was created.
     */
    readonly createdDate: pulumi.Output<string>;
    /**
     * Description of the client certificate.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Date when the client certificate will expire.
     */
    readonly expirationDate: pulumi.Output<string>;
    /**
     * The PEM-encoded public key of the client certificate.
     */
    readonly pemEncodedCertificate: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a ClientCertificate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ClientCertificateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ClientCertificate resources.
 */
export interface ClientCertificateState {
    /**
     * ARN
     */
    arn?: pulumi.Input<string>;
    /**
     * Date when the client certificate was created.
     */
    createdDate?: pulumi.Input<string>;
    /**
     * Description of the client certificate.
     */
    description?: pulumi.Input<string>;
    /**
     * Date when the client certificate will expire.
     */
    expirationDate?: pulumi.Input<string>;
    /**
     * The PEM-encoded public key of the client certificate.
     */
    pemEncodedCertificate?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a ClientCertificate resource.
 */
export interface ClientCertificateArgs {
    /**
     * Description of the client certificate.
     */
    description?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
