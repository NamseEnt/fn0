import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Lightsail Instance. Use this resource to create easy virtual private servers with custom software already setup.
 *
 * > **Note:** Lightsail is currently only supported in a limited number of AWS Regions, please see ["Regions and Availability Zones in Amazon Lightsail"](https://lightsail.aws.amazon.com/ls/docs/overview/article/understanding-regions-and-availability-zones-in-amazon-lightsail) for more details
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lightsail.Instance("example", {
 *     name: "example",
 *     availabilityZone: "us-east-1b",
 *     blueprintId: "amazon_linux_2",
 *     bundleId: "nano_3_0",
 *     keyPairName: "some_key_name",
 *     tags: {
 *         foo: "bar",
 *     },
 * });
 * ```
 *
 * ### Example With User Data
 *
 * Lightsail user data is handled differently than EC2 user data. Lightsail user data only accepts a single lined string. The below example shows installing apache and creating the index page.
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lightsail.Instance("example", {
 *     name: "example",
 *     availabilityZone: "us-east-1b",
 *     blueprintId: "amazon_linux_2",
 *     bundleId: "nano_3_0",
 *     userData: "sudo yum install -y httpd && sudo systemctl start httpd && sudo systemctl enable httpd && echo '<h1>Deployed via Pulumi</h1>' | sudo tee /var/www/html/index.html",
 * });
 * ```
 *
 * ### Enable Auto Snapshots
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lightsail.Instance("example", {
 *     name: "example",
 *     availabilityZone: "us-east-1b",
 *     blueprintId: "amazon_linux_2",
 *     bundleId: "nano_3_0",
 *     addOn: {
 *         type: "AutoSnapshot",
 *         snapshotTime: "06:00",
 *         status: "Enabled",
 *     },
 *     tags: {
 *         foo: "bar",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Lightsail Instances using their name. For example:
 *
 * ```sh
 * $ pulumi import aws:lightsail/instance:Instance example 'example'
 * ```
 */
export declare class Instance extends pulumi.CustomResource {
    /**
     * Get an existing Instance resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: InstanceState, opts?: pulumi.CustomResourceOptions): Instance;
    /**
     * Returns true if the given object is an instance of Instance.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Instance;
    /**
     * Add-on configuration for the instance. See below.
     */
    readonly addOn: pulumi.Output<outputs.lightsail.InstanceAddOn | undefined>;
    /**
     * ARN of the Lightsail instance (matches `id`).
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Availability Zone in which to create your instance. A list of available zones can be obtained using the AWS CLI command: [`aws lightsail get-regions --include-availability-zones`](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/lightsail/get-regions.html).
     */
    readonly availabilityZone: pulumi.Output<string>;
    /**
     * ID for a virtual private server image. A list of available blueprint IDs can be obtained using the AWS CLI command: [`aws lightsail get-blueprints`](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/lightsail/get-blueprints.html).
     */
    readonly blueprintId: pulumi.Output<string>;
    /**
     * Bundle of specification information. A list of available bundle IDs can be obtained using the AWS CLI command: [`aws lightsail get-bundles`](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/lightsail/get-bundles.html).
     */
    readonly bundleId: pulumi.Output<string>;
    /**
     * Number of vCPUs the instance has.
     */
    readonly cpuCount: pulumi.Output<number>;
    /**
     * Timestamp when the instance was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * IP address type of the Lightsail Instance. Valid values: `dualstack`, `ipv4`, `ipv6`. Default: `dualstack`.
     */
    readonly ipAddressType: pulumi.Output<string | undefined>;
    /**
     * List of IPv6 addresses for the Lightsail instance.
     */
    readonly ipv6Addresses: pulumi.Output<string[]>;
    /**
     * Whether this instance has a static IP assigned to it.
     */
    readonly isStaticIp: pulumi.Output<boolean>;
    /**
     * Name of your key pair. Created in the Lightsail console (cannot use `aws.ec2.KeyPair` at this time).
     */
    readonly keyPairName: pulumi.Output<string | undefined>;
    /**
     * Name of the Lightsail Instance. Names must be unique within each AWS Region in your Lightsail account.
     *
     * The following arguments are optional:
     */
    readonly name: pulumi.Output<string>;
    /**
     * Private IP address of the instance.
     */
    readonly privateIpAddress: pulumi.Output<string>;
    /**
     * Public IP address of the instance.
     */
    readonly publicIpAddress: pulumi.Output<string>;
    /**
     * Amount of RAM in GB on the instance (e.g., 1.0).
     */
    readonly ramSize: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
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
     * Single lined launch script as a string to configure server with additional user data.
     */
    readonly userData: pulumi.Output<string | undefined>;
    /**
     * User name for connecting to the instance (e.g., ec2-user).
     */
    readonly username: pulumi.Output<string>;
    /**
     * Create a Instance resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: InstanceArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Instance resources.
 */
export interface InstanceState {
    /**
     * Add-on configuration for the instance. See below.
     */
    addOn?: pulumi.Input<inputs.lightsail.InstanceAddOn>;
    /**
     * ARN of the Lightsail instance (matches `id`).
     */
    arn?: pulumi.Input<string>;
    /**
     * Availability Zone in which to create your instance. A list of available zones can be obtained using the AWS CLI command: [`aws lightsail get-regions --include-availability-zones`](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/lightsail/get-regions.html).
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * ID for a virtual private server image. A list of available blueprint IDs can be obtained using the AWS CLI command: [`aws lightsail get-blueprints`](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/lightsail/get-blueprints.html).
     */
    blueprintId?: pulumi.Input<string>;
    /**
     * Bundle of specification information. A list of available bundle IDs can be obtained using the AWS CLI command: [`aws lightsail get-bundles`](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/lightsail/get-bundles.html).
     */
    bundleId?: pulumi.Input<string>;
    /**
     * Number of vCPUs the instance has.
     */
    cpuCount?: pulumi.Input<number>;
    /**
     * Timestamp when the instance was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * IP address type of the Lightsail Instance. Valid values: `dualstack`, `ipv4`, `ipv6`. Default: `dualstack`.
     */
    ipAddressType?: pulumi.Input<string>;
    /**
     * List of IPv6 addresses for the Lightsail instance.
     */
    ipv6Addresses?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Whether this instance has a static IP assigned to it.
     */
    isStaticIp?: pulumi.Input<boolean>;
    /**
     * Name of your key pair. Created in the Lightsail console (cannot use `aws.ec2.KeyPair` at this time).
     */
    keyPairName?: pulumi.Input<string>;
    /**
     * Name of the Lightsail Instance. Names must be unique within each AWS Region in your Lightsail account.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Private IP address of the instance.
     */
    privateIpAddress?: pulumi.Input<string>;
    /**
     * Public IP address of the instance.
     */
    publicIpAddress?: pulumi.Input<string>;
    /**
     * Amount of RAM in GB on the instance (e.g., 1.0).
     */
    ramSize?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
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
    /**
     * Single lined launch script as a string to configure server with additional user data.
     */
    userData?: pulumi.Input<string>;
    /**
     * User name for connecting to the instance (e.g., ec2-user).
     */
    username?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Instance resource.
 */
export interface InstanceArgs {
    /**
     * Add-on configuration for the instance. See below.
     */
    addOn?: pulumi.Input<inputs.lightsail.InstanceAddOn>;
    /**
     * Availability Zone in which to create your instance. A list of available zones can be obtained using the AWS CLI command: [`aws lightsail get-regions --include-availability-zones`](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/lightsail/get-regions.html).
     */
    availabilityZone: pulumi.Input<string>;
    /**
     * ID for a virtual private server image. A list of available blueprint IDs can be obtained using the AWS CLI command: [`aws lightsail get-blueprints`](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/lightsail/get-blueprints.html).
     */
    blueprintId: pulumi.Input<string>;
    /**
     * Bundle of specification information. A list of available bundle IDs can be obtained using the AWS CLI command: [`aws lightsail get-bundles`](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/lightsail/get-bundles.html).
     */
    bundleId: pulumi.Input<string>;
    /**
     * IP address type of the Lightsail Instance. Valid values: `dualstack`, `ipv4`, `ipv6`. Default: `dualstack`.
     */
    ipAddressType?: pulumi.Input<string>;
    /**
     * Name of your key pair. Created in the Lightsail console (cannot use `aws.ec2.KeyPair` at this time).
     */
    keyPairName?: pulumi.Input<string>;
    /**
     * Name of the Lightsail Instance. Names must be unique within each AWS Region in your Lightsail account.
     *
     * The following arguments are optional:
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the resource. To create a key-only tag, use an empty string as the value. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Single lined launch script as a string to configure server with additional user data.
     */
    userData?: pulumi.Input<string>;
}
