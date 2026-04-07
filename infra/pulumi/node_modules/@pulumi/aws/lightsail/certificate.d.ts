import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Lightsail certificate. Use this resource to create and manage SSL/TLS certificates for securing custom domains with your Lightsail resources.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lightsail.Certificate("example", {
 *     name: "example-certificate",
 *     domainName: "example.com",
 *     subjectAlternativeNames: ["www.example.com"],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_lightsail_certificate` using the certificate name. For example:
 *
 * ```sh
 * $ pulumi import aws:lightsail/certificate:Certificate example example-certificate
 * ```
 */
export declare class Certificate extends pulumi.CustomResource {
    /**
     * Get an existing Certificate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: CertificateState, opts?: pulumi.CustomResourceOptions): Certificate;
    /**
     * Returns true if the given object is an instance of Certificate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Certificate;
    /**
     * ARN of the certificate.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Date and time when the certificate was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Domain name for which the certificate should be issued.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * Set of domain validation objects which can be used to complete certificate validation. Can have more than one element, e.g., if SANs are defined. Each element contains the following attributes:
     */
    readonly domainValidationOptions: pulumi.Output<outputs.lightsail.CertificateDomainValidationOption[]>;
    /**
     * Name of the certificate.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Set of domains that should be SANs in the issued certificate. `domainName` attribute is automatically added as a Subject Alternative Name.
     */
    readonly subjectAlternativeNames: pulumi.Output<string[]>;
    /**
     * Map of tags to assign to the resource. To create a key-only tag, use an empty string as the value. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Certificate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: CertificateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Certificate resources.
 */
export interface CertificateState {
    /**
     * ARN of the certificate.
     */
    arn?: pulumi.Input<string>;
    /**
     * Date and time when the certificate was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Domain name for which the certificate should be issued.
     */
    domainName?: pulumi.Input<string>;
    /**
     * Set of domain validation objects which can be used to complete certificate validation. Can have more than one element, e.g., if SANs are defined. Each element contains the following attributes:
     */
    domainValidationOptions?: pulumi.Input<pulumi.Input<inputs.lightsail.CertificateDomainValidationOption>[]>;
    /**
     * Name of the certificate.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Set of domains that should be SANs in the issued certificate. `domainName` attribute is automatically added as a Subject Alternative Name.
     */
    subjectAlternativeNames?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Map of tags to assign to the resource. To create a key-only tag, use an empty string as the value. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Certificate resource.
 */
export interface CertificateArgs {
    /**
     * Domain name for which the certificate should be issued.
     */
    domainName?: pulumi.Input<string>;
    /**
     * Name of the certificate.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Set of domains that should be SANs in the issued certificate. `domainName` attribute is automatically added as a Subject Alternative Name.
     */
    subjectAlternativeNames?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Map of tags to assign to the resource. To create a key-only tag, use an empty string as the value. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
