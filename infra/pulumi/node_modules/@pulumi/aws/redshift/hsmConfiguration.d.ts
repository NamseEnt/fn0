import * as pulumi from "@pulumi/pulumi";
/**
 * Creates an HSM configuration that contains the information required by an Amazon Redshift cluster to store and use database encryption keys in a Hardware Security Module (HSM).
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshift.HsmConfiguration("example", {
 *     description: "example",
 *     hsmConfigurationIdentifier: "example",
 *     hsmIpAddress: "10.0.0.1",
 *     hsmPartitionName: "aws",
 *     hsmPartitionPassword: "example",
 *     hsmServerPublicCertificate: "example",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift HSM Client Certificates using `hsm_configuration_identifier`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshift/hsmConfiguration:HsmConfiguration example example
 * ```
 */
export declare class HsmConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing HsmConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: HsmConfigurationState, opts?: pulumi.CustomResourceOptions): HsmConfiguration;
    /**
     * Returns true if the given object is an instance of HsmConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is HsmConfiguration;
    /**
     * Amazon Resource Name (ARN) of the Hsm Client Certificate.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * A text description of the HSM configuration to be created.
     */
    readonly description: pulumi.Output<string>;
    /**
     * The identifier to be assigned to the new Amazon Redshift HSM configuration.
     */
    readonly hsmConfigurationIdentifier: pulumi.Output<string>;
    /**
     * The IP address that the Amazon Redshift cluster must use to access the HSM.
     */
    readonly hsmIpAddress: pulumi.Output<string>;
    /**
     * The name of the partition in the HSM where the Amazon Redshift clusters will store their database encryption keys.
     */
    readonly hsmPartitionName: pulumi.Output<string>;
    /**
     * The password required to access the HSM partition.
     */
    readonly hsmPartitionPassword: pulumi.Output<string>;
    /**
     * The HSMs public certificate file. When using Cloud HSM, the file name is server.pem.
     */
    readonly hsmServerPublicCertificate: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a HsmConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: HsmConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering HsmConfiguration resources.
 */
export interface HsmConfigurationState {
    /**
     * Amazon Resource Name (ARN) of the Hsm Client Certificate.
     */
    arn?: pulumi.Input<string>;
    /**
     * A text description of the HSM configuration to be created.
     */
    description?: pulumi.Input<string>;
    /**
     * The identifier to be assigned to the new Amazon Redshift HSM configuration.
     */
    hsmConfigurationIdentifier?: pulumi.Input<string>;
    /**
     * The IP address that the Amazon Redshift cluster must use to access the HSM.
     */
    hsmIpAddress?: pulumi.Input<string>;
    /**
     * The name of the partition in the HSM where the Amazon Redshift clusters will store their database encryption keys.
     */
    hsmPartitionName?: pulumi.Input<string>;
    /**
     * The password required to access the HSM partition.
     */
    hsmPartitionPassword?: pulumi.Input<string>;
    /**
     * The HSMs public certificate file. When using Cloud HSM, the file name is server.pem.
     */
    hsmServerPublicCertificate?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * A map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a HsmConfiguration resource.
 */
export interface HsmConfigurationArgs {
    /**
     * A text description of the HSM configuration to be created.
     */
    description: pulumi.Input<string>;
    /**
     * The identifier to be assigned to the new Amazon Redshift HSM configuration.
     */
    hsmConfigurationIdentifier: pulumi.Input<string>;
    /**
     * The IP address that the Amazon Redshift cluster must use to access the HSM.
     */
    hsmIpAddress: pulumi.Input<string>;
    /**
     * The name of the partition in the HSM where the Amazon Redshift clusters will store their database encryption keys.
     */
    hsmPartitionName: pulumi.Input<string>;
    /**
     * The password required to access the HSM partition.
     */
    hsmPartitionPassword: pulumi.Input<string>;
    /**
     * The HSMs public certificate file. When using Cloud HSM, the file name is server.pem.
     */
    hsmServerPublicCertificate: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
