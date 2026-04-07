import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS FinSpace Kx Environment.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kms.Key("example", {
 *     description: "Sample KMS Key",
 *     deletionWindowInDays: 7,
 * });
 * const exampleKxEnvironment = new aws.finspace.KxEnvironment("example", {
 *     name: "my-tf-kx-environment",
 *     kmsKeyId: example.arn,
 * });
 * ```
 *
 * ### With Transit Gateway Configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kms.Key("example", {
 *     description: "Sample KMS Key",
 *     deletionWindowInDays: 7,
 * });
 * const exampleTransitGateway = new aws.ec2transitgateway.TransitGateway("example", {description: "example"});
 * const exampleEnv = new aws.finspace.KxEnvironment("example_env", {
 *     name: "my-tf-kx-environment",
 *     description: "Environment description",
 *     kmsKeyId: example.arn,
 *     transitGatewayConfiguration: {
 *         transitGatewayId: exampleTransitGateway.id,
 *         routableCidrSpace: "100.64.0.0/26",
 *     },
 *     customDnsConfigurations: [{
 *         customDnsServerName: "example.finspace.amazonaws.com",
 *         customDnsServerIp: "10.0.0.76",
 *     }],
 * });
 * ```
 *
 * ### With Transit Gateway Attachment Network ACL Configuration
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.kms.Key("example", {
 *     description: "Sample KMS Key",
 *     deletionWindowInDays: 7,
 * });
 * const exampleTransitGateway = new aws.ec2transitgateway.TransitGateway("example", {description: "example"});
 * const exampleEnv = new aws.finspace.KxEnvironment("example_env", {
 *     name: "my-tf-kx-environment",
 *     description: "Environment description",
 *     kmsKeyId: example.arn,
 *     transitGatewayConfiguration: {
 *         transitGatewayId: exampleTransitGateway.id,
 *         routableCidrSpace: "100.64.0.0/26",
 *         attachmentNetworkAclConfigurations: [{
 *             ruleNumber: 1,
 *             protocol: "6",
 *             ruleAction: "allow",
 *             cidrBlock: "0.0.0.0/0",
 *             portRange: {
 *                 from: 53,
 *                 to: 53,
 *             },
 *             icmpTypeCode: {
 *                 type: -1,
 *                 code: -1,
 *             },
 *         }],
 *     },
 *     customDnsConfigurations: [{
 *         customDnsServerName: "example.finspace.amazonaws.com",
 *         customDnsServerIp: "10.0.0.76",
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import an AWS FinSpace Kx Environment using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:finspace/kxEnvironment:KxEnvironment example n3ceo7wqxoxcti5tujqwzs
 * ```
 */
export declare class KxEnvironment extends pulumi.CustomResource {
    /**
     * Get an existing KxEnvironment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: KxEnvironmentState, opts?: pulumi.CustomResourceOptions): KxEnvironment;
    /**
     * Returns true if the given object is an instance of KxEnvironment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is KxEnvironment;
    /**
     * Amazon Resource Name (ARN) identifier of the KX environment.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * AWS Availability Zone IDs that this environment is available in. Important when selecting VPC subnets to use in cluster creation.
     */
    readonly availabilityZones: pulumi.Output<string[]>;
    /**
     * Timestamp at which the environment is created in FinSpace. Value determined as epoch time in seconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000.
     */
    readonly createdTimestamp: pulumi.Output<string>;
    /**
     * List of DNS server name and server IP. This is used to set up Route-53 outbound resolvers. Defined below.
     */
    readonly customDnsConfigurations: pulumi.Output<outputs.finspace.KxEnvironmentCustomDnsConfiguration[] | undefined>;
    /**
     * Description for the KX environment.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Unique identifier for the AWS environment infrastructure account.
     */
    readonly infrastructureAccountId: pulumi.Output<string>;
    /**
     * KMS key ID to encrypt your data in the FinSpace environment.
     *
     * The following arguments are optional:
     */
    readonly kmsKeyId: pulumi.Output<string>;
    /**
     * Last timestamp at which the environment was updated in FinSpace. Value determined as epoch time in seconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000.
     */
    readonly lastModifiedTimestamp: pulumi.Output<string>;
    /**
     * Name of the KX environment that you want to create.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Status of environment creation
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
     * Transit gateway and network configuration that is used to connect the KX environment to an internal network. Defined below.
     */
    readonly transitGatewayConfiguration: pulumi.Output<outputs.finspace.KxEnvironmentTransitGatewayConfiguration | undefined>;
    /**
     * Create a KxEnvironment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: KxEnvironmentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering KxEnvironment resources.
 */
export interface KxEnvironmentState {
    /**
     * Amazon Resource Name (ARN) identifier of the KX environment.
     */
    arn?: pulumi.Input<string>;
    /**
     * AWS Availability Zone IDs that this environment is available in. Important when selecting VPC subnets to use in cluster creation.
     */
    availabilityZones?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Timestamp at which the environment is created in FinSpace. Value determined as epoch time in seconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000.
     */
    createdTimestamp?: pulumi.Input<string>;
    /**
     * List of DNS server name and server IP. This is used to set up Route-53 outbound resolvers. Defined below.
     */
    customDnsConfigurations?: pulumi.Input<pulumi.Input<inputs.finspace.KxEnvironmentCustomDnsConfiguration>[]>;
    /**
     * Description for the KX environment.
     */
    description?: pulumi.Input<string>;
    /**
     * Unique identifier for the AWS environment infrastructure account.
     */
    infrastructureAccountId?: pulumi.Input<string>;
    /**
     * KMS key ID to encrypt your data in the FinSpace environment.
     *
     * The following arguments are optional:
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Last timestamp at which the environment was updated in FinSpace. Value determined as epoch time in seconds. For example, the value for Monday, November 1, 2021 12:00:00 PM UTC is specified as 1635768000.
     */
    lastModifiedTimestamp?: pulumi.Input<string>;
    /**
     * Name of the KX environment that you want to create.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Status of environment creation
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
    /**
     * Transit gateway and network configuration that is used to connect the KX environment to an internal network. Defined below.
     */
    transitGatewayConfiguration?: pulumi.Input<inputs.finspace.KxEnvironmentTransitGatewayConfiguration>;
}
/**
 * The set of arguments for constructing a KxEnvironment resource.
 */
export interface KxEnvironmentArgs {
    /**
     * List of DNS server name and server IP. This is used to set up Route-53 outbound resolvers. Defined below.
     */
    customDnsConfigurations?: pulumi.Input<pulumi.Input<inputs.finspace.KxEnvironmentCustomDnsConfiguration>[]>;
    /**
     * Description for the KX environment.
     */
    description?: pulumi.Input<string>;
    /**
     * KMS key ID to encrypt your data in the FinSpace environment.
     *
     * The following arguments are optional:
     */
    kmsKeyId: pulumi.Input<string>;
    /**
     * Name of the KX environment that you want to create.
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
    /**
     * Transit gateway and network configuration that is used to connect the KX environment to an internal network. Defined below.
     */
    transitGatewayConfiguration?: pulumi.Input<inputs.finspace.KxEnvironmentTransitGatewayConfiguration>;
}
