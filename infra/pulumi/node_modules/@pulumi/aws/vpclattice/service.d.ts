import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS VPC Lattice Service.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.vpclattice.Service("example", {
 *     name: "example",
 *     authType: "AWS_IAM",
 *     customDomainName: "example.com",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC Lattice Service using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:vpclattice/service:Service example svc-06728e2357ea55f8a
 * ```
 */
export declare class Service extends pulumi.CustomResource {
    /**
     * Get an existing Service resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ServiceState, opts?: pulumi.CustomResourceOptions): Service;
    /**
     * Returns true if the given object is an instance of Service.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Service;
    /**
     * ARN of the service.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Type of IAM policy. Either `NONE` or `AWS_IAM`.
     */
    readonly authType: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the certificate.
     */
    readonly certificateArn: pulumi.Output<string | undefined>;
    /**
     * Custom domain name of the service.
     */
    readonly customDomainName: pulumi.Output<string | undefined>;
    /**
     * DNS name of the service.
     */
    readonly dnsEntries: pulumi.Output<outputs.vpclattice.ServiceDnsEntry[]>;
    /**
     * Name of the service. The name must be unique within the account. The valid characters are a-z, 0-9, and hyphens (-). You can't use a hyphen as the first or last character, or immediately after another hyphen.Must be between 3 and 40 characters in length.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Status of the service.
     */
    readonly status: pulumi.Output<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Service resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: ServiceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Service resources.
 */
export interface ServiceState {
    /**
     * ARN of the service.
     */
    arn?: pulumi.Input<string>;
    /**
     * Type of IAM policy. Either `NONE` or `AWS_IAM`.
     */
    authType?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the certificate.
     */
    certificateArn?: pulumi.Input<string>;
    /**
     * Custom domain name of the service.
     */
    customDomainName?: pulumi.Input<string>;
    /**
     * DNS name of the service.
     */
    dnsEntries?: pulumi.Input<pulumi.Input<inputs.vpclattice.ServiceDnsEntry>[]>;
    /**
     * Name of the service. The name must be unique within the account. The valid characters are a-z, 0-9, and hyphens (-). You can't use a hyphen as the first or last character, or immediately after another hyphen.Must be between 3 and 40 characters in length.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Status of the service.
     */
    status?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Service resource.
 */
export interface ServiceArgs {
    /**
     * Type of IAM policy. Either `NONE` or `AWS_IAM`.
     */
    authType?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the certificate.
     */
    certificateArn?: pulumi.Input<string>;
    /**
     * Custom domain name of the service.
     */
    customDomainName?: pulumi.Input<string>;
    /**
     * Name of the service. The name must be unique within the account. The valid characters are a-z, 0-9, and hyphens (-). You can't use a hyphen as the first or last character, or immediately after another hyphen.Must be between 3 and 40 characters in length.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value mapping of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
