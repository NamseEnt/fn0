import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS QuickSight VPC Connection.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const vpcConnectionRole = new aws.iam.Role("vpc_connection_role", {
 *     assumeRolePolicy: JSON.stringify({
 *         Version: "2012-10-17",
 *         Statement: [{
 *             Effect: "Allow",
 *             Action: "sts:AssumeRole",
 *             Principal: {
 *                 Service: "quicksight.amazonaws.com",
 *             },
 *         }],
 *     }),
 *     inlinePolicies: [{
 *         name: "QuickSightVPCConnectionRolePolicy",
 *         policy: JSON.stringify({
 *             Version: "2012-10-17",
 *             Statement: [{
 *                 Effect: "Allow",
 *                 Action: [
 *                     "ec2:CreateNetworkInterface",
 *                     "ec2:ModifyNetworkInterfaceAttribute",
 *                     "ec2:DeleteNetworkInterface",
 *                     "ec2:DescribeSubnets",
 *                     "ec2:DescribeSecurityGroups",
 *                 ],
 *                 Resource: ["*"],
 *             }],
 *         }),
 *     }],
 * });
 * const example = new aws.quicksight.VpcConnection("example", {
 *     vpcConnectionId: "example-connection-id",
 *     name: "Example Connection",
 *     roleArn: vpcConnectionRole.arn,
 *     securityGroupIds: ["sg-00000000000000000"],
 *     subnetIds: [
 *         "subnet-00000000000000000",
 *         "subnet-00000000000000001",
 *     ],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import QuickSight VPC connection using the AWS account ID and VPC connection ID separated by commas (`,`). For example:
 *
 * ```sh
 * $ pulumi import aws:quicksight/vpcConnection:VpcConnection example 123456789012,example
 * ```
 */
export declare class VpcConnection extends pulumi.CustomResource {
    /**
     * Get an existing VpcConnection resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcConnectionState, opts?: pulumi.CustomResourceOptions): VpcConnection;
    /**
     * Returns true if the given object is an instance of VpcConnection.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpcConnection;
    /**
     * ARN of the VPC connection.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The availability status of the VPC connection. Valid values are `AVAILABLE`, `UNAVAILABLE` or `PARTIALLY_AVAILABLE`.
     */
    readonly availabilityStatus: pulumi.Output<string>;
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * A list of IP addresses of DNS resolver endpoints for the VPC connection.
     */
    readonly dnsResolvers: pulumi.Output<string[] | undefined>;
    /**
     * The display name for the VPC connection.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The IAM role to associate with the VPC connection.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * A list of security group IDs for the VPC connection.
     */
    readonly securityGroupIds: pulumi.Output<string[]>;
    /**
     * A list of subnet IDs for the VPC connection.
     *
     * The following arguments are optional:
     */
    readonly subnetIds: pulumi.Output<string[]>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    readonly timeouts: pulumi.Output<outputs.quicksight.VpcConnectionTimeouts | undefined>;
    /**
     * The ID of the VPC connection.
     */
    readonly vpcConnectionId: pulumi.Output<string>;
    /**
     * Create a VpcConnection resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpcConnectionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpcConnection resources.
 */
export interface VpcConnectionState {
    /**
     * ARN of the VPC connection.
     */
    arn?: pulumi.Input<string>;
    /**
     * The availability status of the VPC connection. Valid values are `AVAILABLE`, `UNAVAILABLE` or `PARTIALLY_AVAILABLE`.
     */
    availabilityStatus?: pulumi.Input<string>;
    awsAccountId?: pulumi.Input<string>;
    /**
     * A list of IP addresses of DNS resolver endpoints for the VPC connection.
     */
    dnsResolvers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The display name for the VPC connection.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IAM role to associate with the VPC connection.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * A list of security group IDs for the VPC connection.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A list of subnet IDs for the VPC connection.
     *
     * The following arguments are optional:
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
    timeouts?: pulumi.Input<inputs.quicksight.VpcConnectionTimeouts>;
    /**
     * The ID of the VPC connection.
     */
    vpcConnectionId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a VpcConnection resource.
 */
export interface VpcConnectionArgs {
    awsAccountId?: pulumi.Input<string>;
    /**
     * A list of IP addresses of DNS resolver endpoints for the VPC connection.
     */
    dnsResolvers?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The display name for the VPC connection.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IAM role to associate with the VPC connection.
     */
    roleArn: pulumi.Input<string>;
    /**
     * A list of security group IDs for the VPC connection.
     */
    securityGroupIds: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * A list of subnet IDs for the VPC connection.
     *
     * The following arguments are optional:
     */
    subnetIds: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.quicksight.VpcConnectionTimeouts>;
    /**
     * The ID of the VPC connection.
     */
    vpcConnectionId: pulumi.Input<string>;
}
