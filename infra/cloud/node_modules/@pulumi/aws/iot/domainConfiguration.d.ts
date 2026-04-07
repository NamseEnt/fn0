import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Creates and manages an AWS IoT domain configuration.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const iot = new aws.iot.DomainConfiguration("iot", {
 *     name: "iot-",
 *     domainName: "iot.example.com",
 *     serviceType: "DATA",
 *     serverCertificateArns: [cert.arn],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import domain configurations using the name. For example:
 *
 * ```sh
 * $ pulumi import aws:iot/domainConfiguration:DomainConfiguration example example
 * ```
 */
export declare class DomainConfiguration extends pulumi.CustomResource {
    /**
     * Get an existing DomainConfiguration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DomainConfigurationState, opts?: pulumi.CustomResourceOptions): DomainConfiguration;
    /**
     * Returns true if the given object is an instance of DomainConfiguration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DomainConfiguration;
    /**
     * An enumerated string that speciﬁes the application-layer protocol. Valid values are `SECURE_MQTT`, `MQTT_WSS`, `HTTPS` or `DEFAULT`.
     */
    readonly applicationProtocol: pulumi.Output<string>;
    /**
     * The ARN of the domain configuration.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * An enumerated string that speciﬁes the authentication type. Valid values are `CUSTOM_AUTH_X509`, `CUSTOM_AUTH`, `AWS_X509`, `AWS_SIGV4` or `DEFAULT`.
     */
    readonly authenticationType: pulumi.Output<string>;
    /**
     * An object that specifies the authorization service for a domain. See the `authorizerConfig` Block below for details.
     */
    readonly authorizerConfig: pulumi.Output<outputs.iot.DomainConfigurationAuthorizerConfig | undefined>;
    /**
     * Fully-qualified domain name.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * The type of the domain.
     */
    readonly domainType: pulumi.Output<string>;
    /**
     * The name of the domain configuration. This value must be unique to a region.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ARNs of the certificates that IoT passes to the device during the TLS handshake. Currently you can specify only one certificate ARN. This value is not required for Amazon Web Services-managed domains. When using a custom `domainName`, the cert must include it.
     */
    readonly serverCertificateArns: pulumi.Output<string[] | undefined>;
    /**
     * The type of service delivered by the endpoint. Note: Amazon Web Services IoT Core currently supports only the `DATA` service type.
     */
    readonly serviceType: pulumi.Output<string | undefined>;
    /**
     * The status to which the domain configuration should be set. Valid values are `ENABLED` and `DISABLED`.
     */
    readonly status: pulumi.Output<string | undefined>;
    /**
     * Map of tags to assign to this resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * An object that specifies the TLS configuration for a domain. See the `tlsConfig` Block below for details.
     */
    readonly tlsConfig: pulumi.Output<outputs.iot.DomainConfigurationTlsConfig>;
    /**
     * The certificate used to validate the server certificate and prove domain name ownership. This certificate must be signed by a public certificate authority. This value is not required for Amazon Web Services-managed domains.
     */
    readonly validationCertificateArn: pulumi.Output<string | undefined>;
    /**
     * Create a DomainConfiguration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: DomainConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DomainConfiguration resources.
 */
export interface DomainConfigurationState {
    /**
     * An enumerated string that speciﬁes the application-layer protocol. Valid values are `SECURE_MQTT`, `MQTT_WSS`, `HTTPS` or `DEFAULT`.
     */
    applicationProtocol?: pulumi.Input<string>;
    /**
     * The ARN of the domain configuration.
     */
    arn?: pulumi.Input<string>;
    /**
     * An enumerated string that speciﬁes the authentication type. Valid values are `CUSTOM_AUTH_X509`, `CUSTOM_AUTH`, `AWS_X509`, `AWS_SIGV4` or `DEFAULT`.
     */
    authenticationType?: pulumi.Input<string>;
    /**
     * An object that specifies the authorization service for a domain. See the `authorizerConfig` Block below for details.
     */
    authorizerConfig?: pulumi.Input<inputs.iot.DomainConfigurationAuthorizerConfig>;
    /**
     * Fully-qualified domain name.
     */
    domainName?: pulumi.Input<string>;
    /**
     * The type of the domain.
     */
    domainType?: pulumi.Input<string>;
    /**
     * The name of the domain configuration. This value must be unique to a region.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARNs of the certificates that IoT passes to the device during the TLS handshake. Currently you can specify only one certificate ARN. This value is not required for Amazon Web Services-managed domains. When using a custom `domainName`, the cert must include it.
     */
    serverCertificateArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The type of service delivered by the endpoint. Note: Amazon Web Services IoT Core currently supports only the `DATA` service type.
     */
    serviceType?: pulumi.Input<string>;
    /**
     * The status to which the domain configuration should be set. Valid values are `ENABLED` and `DISABLED`.
     */
    status?: pulumi.Input<string>;
    /**
     * Map of tags to assign to this resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    /**
     * An object that specifies the TLS configuration for a domain. See the `tlsConfig` Block below for details.
     */
    tlsConfig?: pulumi.Input<inputs.iot.DomainConfigurationTlsConfig>;
    /**
     * The certificate used to validate the server certificate and prove domain name ownership. This certificate must be signed by a public certificate authority. This value is not required for Amazon Web Services-managed domains.
     */
    validationCertificateArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DomainConfiguration resource.
 */
export interface DomainConfigurationArgs {
    /**
     * An enumerated string that speciﬁes the application-layer protocol. Valid values are `SECURE_MQTT`, `MQTT_WSS`, `HTTPS` or `DEFAULT`.
     */
    applicationProtocol?: pulumi.Input<string>;
    /**
     * An enumerated string that speciﬁes the authentication type. Valid values are `CUSTOM_AUTH_X509`, `CUSTOM_AUTH`, `AWS_X509`, `AWS_SIGV4` or `DEFAULT`.
     */
    authenticationType?: pulumi.Input<string>;
    /**
     * An object that specifies the authorization service for a domain. See the `authorizerConfig` Block below for details.
     */
    authorizerConfig?: pulumi.Input<inputs.iot.DomainConfigurationAuthorizerConfig>;
    /**
     * Fully-qualified domain name.
     */
    domainName?: pulumi.Input<string>;
    /**
     * The name of the domain configuration. This value must be unique to a region.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ARNs of the certificates that IoT passes to the device during the TLS handshake. Currently you can specify only one certificate ARN. This value is not required for Amazon Web Services-managed domains. When using a custom `domainName`, the cert must include it.
     */
    serverCertificateArns?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The type of service delivered by the endpoint. Note: Amazon Web Services IoT Core currently supports only the `DATA` service type.
     */
    serviceType?: pulumi.Input<string>;
    /**
     * The status to which the domain configuration should be set. Valid values are `ENABLED` and `DISABLED`.
     */
    status?: pulumi.Input<string>;
    /**
     * Map of tags to assign to this resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * An object that specifies the TLS configuration for a domain. See the `tlsConfig` Block below for details.
     */
    tlsConfig?: pulumi.Input<inputs.iot.DomainConfigurationTlsConfig>;
    /**
     * The certificate used to validate the server certificate and prove domain name ownership. This certificate must be signed by a public certificate authority. This value is not required for Amazon Web Services-managed domains.
     */
    validationCertificateArn?: pulumi.Input<string>;
}
