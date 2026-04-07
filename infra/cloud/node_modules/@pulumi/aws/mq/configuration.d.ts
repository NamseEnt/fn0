import * as pulumi from "@pulumi/pulumi";
/**
 * Manages an Amazon MQ configuration. Use this resource to create and manage broker configurations for ActiveMQ and RabbitMQ brokers.
 *
 * ## Example Usage
 *
 * ### ActiveMQ
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.mq.Configuration("example", {
 *     description: "Example Configuration",
 *     name: "example",
 *     engineType: "ActiveMQ",
 *     engineVersion: "5.17.6",
 *     data: `<?xml version=\\"1.0\\" encoding=\\"UTF-8\\" standalone=\\"yes\\"?>
 * <broker xmlns=\\"http://activemq.apache.org/schema/core\\">
 *   <plugins>
 *     <forcePersistencyModeBrokerPlugin persistenceFlag=\\"true\\"/>
 *     <statisticsBrokerPlugin/>
 *     <timeStampingBrokerPlugin ttlCeiling=\\"86400000\\" zeroExpirationOverride=\\"86400000\\"/>
 *   </plugins>
 * </broker>
 * `,
 * });
 * ```
 *
 * ### RabbitMQ
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.mq.Configuration("example", {
 *     description: "Example Configuration",
 *     name: "example",
 *     engineType: "RabbitMQ",
 *     engineVersion: "3.11.20",
 *     data: `# Default RabbitMQ delivery acknowledgement timeout is 30 minutes in milliseconds
 * consumer_timeout = 1800000
 * `,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import MQ Configurations using the configuration ID. For example:
 *
 * ```sh
 * $ pulumi import aws:mq/configuration:Configuration example c-0187d1eb-88c8-475a-9b79-16ef5a10c94f
 * ```
 */
export declare class Configuration extends pulumi.CustomResource {
    /**
     * Get an existing Configuration resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ConfigurationState, opts?: pulumi.CustomResourceOptions): Configuration;
    /**
     * Returns true if the given object is an instance of Configuration.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Configuration;
    /**
     * ARN of the configuration.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Authentication strategy associated with the configuration. Valid values are `simple` and `ldap`. `ldap` is not supported for RabbitMQ engine type.
     */
    readonly authenticationStrategy: pulumi.Output<string>;
    /**
     * Broker configuration in XML format for ActiveMQ or Cuttlefish format for RabbitMQ. See [AWS documentation](https://docs.aws.amazon.com/amazon-mq/latest/developer-guide/amazon-mq-broker-configuration-parameters.html) for supported parameters and format of the XML.
     */
    readonly data: pulumi.Output<string>;
    /**
     * Description of the configuration.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Type of broker engine. Valid values are `ActiveMQ` and `RabbitMQ`.
     */
    readonly engineType: pulumi.Output<string>;
    /**
     * Version of the broker engine.
     */
    readonly engineVersion: pulumi.Output<string>;
    /**
     * Latest revision of the configuration.
     */
    readonly latestRevision: pulumi.Output<number>;
    /**
     * Name of the configuration.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Key-value map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Create a Configuration resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ConfigurationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Configuration resources.
 */
export interface ConfigurationState {
    /**
     * ARN of the configuration.
     */
    arn?: pulumi.Input<string>;
    /**
     * Authentication strategy associated with the configuration. Valid values are `simple` and `ldap`. `ldap` is not supported for RabbitMQ engine type.
     */
    authenticationStrategy?: pulumi.Input<string>;
    /**
     * Broker configuration in XML format for ActiveMQ or Cuttlefish format for RabbitMQ. See [AWS documentation](https://docs.aws.amazon.com/amazon-mq/latest/developer-guide/amazon-mq-broker-configuration-parameters.html) for supported parameters and format of the XML.
     */
    data?: pulumi.Input<string>;
    /**
     * Description of the configuration.
     */
    description?: pulumi.Input<string>;
    /**
     * Type of broker engine. Valid values are `ActiveMQ` and `RabbitMQ`.
     */
    engineType?: pulumi.Input<string>;
    /**
     * Version of the broker engine.
     */
    engineVersion?: pulumi.Input<string>;
    /**
     * Latest revision of the configuration.
     */
    latestRevision?: pulumi.Input<number>;
    /**
     * Name of the configuration.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
 * The set of arguments for constructing a Configuration resource.
 */
export interface ConfigurationArgs {
    /**
     * Authentication strategy associated with the configuration. Valid values are `simple` and `ldap`. `ldap` is not supported for RabbitMQ engine type.
     */
    authenticationStrategy?: pulumi.Input<string>;
    /**
     * Broker configuration in XML format for ActiveMQ or Cuttlefish format for RabbitMQ. See [AWS documentation](https://docs.aws.amazon.com/amazon-mq/latest/developer-guide/amazon-mq-broker-configuration-parameters.html) for supported parameters and format of the XML.
     */
    data: pulumi.Input<string>;
    /**
     * Description of the configuration.
     */
    description?: pulumi.Input<string>;
    /**
     * Type of broker engine. Valid values are `ActiveMQ` and `RabbitMQ`.
     */
    engineType: pulumi.Input<string>;
    /**
     * Version of the broker engine.
     */
    engineVersion: pulumi.Input<string>;
    /**
     * Name of the configuration.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Key-value map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
