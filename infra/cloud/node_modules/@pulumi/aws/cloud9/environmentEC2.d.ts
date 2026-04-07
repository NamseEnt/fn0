import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Cloud9 EC2 Development Environment.
 *
 * ## Example Usage
 *
 * Basic usage:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloud9.EnvironmentEC2("example", {
 *     instanceType: "t2.micro",
 *     name: "example-env",
 *     imageId: "amazonlinux-2023-x86_64",
 * });
 * ```
 *
 * Get the URL of the Cloud9 environment after creation:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloud9.EnvironmentEC2("example", {instanceType: "t2.micro"});
 * const cloud9Instance = aws.ec2.getInstanceOutput({
 *     filters: [{
 *         name: "tag:aws:cloud9:environment",
 *         values: [example.id],
 *     }],
 * });
 * export const cloud9Url = pulumi.interpolate`https://${region}.console.aws.amazon.com/cloud9/ide/${example.id}`;
 * ```
 *
 * Allocate a static IP to the Cloud9 environment:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.cloud9.EnvironmentEC2("example", {instanceType: "t2.micro"});
 * const cloud9Instance = aws.ec2.getInstanceOutput({
 *     filters: [{
 *         name: "tag:aws:cloud9:environment",
 *         values: [example.id],
 *     }],
 * });
 * const cloud9Eip = new aws.ec2.Eip("cloud9_eip", {
 *     instance: cloud9Instance.apply(cloud9Instance => cloud9Instance.id),
 *     domain: "vpc",
 * });
 * export const cloud9PublicIp = cloud9Eip.publicIp;
 * ```
 */
export declare class EnvironmentEC2 extends pulumi.CustomResource {
    /**
     * Get an existing EnvironmentEC2 resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EnvironmentEC2State, opts?: pulumi.CustomResourceOptions): EnvironmentEC2;
    /**
     * Returns true if the given object is an instance of EnvironmentEC2.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EnvironmentEC2;
    /**
     * The ARN of the environment.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The number of minutes until the running instance is shut down after the environment has last been used.
     */
    readonly automaticStopTimeMinutes: pulumi.Output<number | undefined>;
    /**
     * The connection type used for connecting to an Amazon EC2 environment. Valid values are `CONNECT_SSH` and `CONNECT_SSM`. For more information please refer [AWS documentation for Cloud9](https://docs.aws.amazon.com/cloud9/latest/user-guide/ec2-ssm.html).
     */
    readonly connectionType: pulumi.Output<string | undefined>;
    /**
     * The description of the environment.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The identifier for the Amazon Machine Image (AMI) that's used to create the EC2 instance. Valid values are
     * * `amazonlinux-2-x86_64`
     * * `amazonlinux-2023-x86_64`
     * * `ubuntu-18.04-x86_64`
     * * `ubuntu-22.04-x86_64`
     * * `resolve:ssm:/aws/service/cloud9/amis/amazonlinux-2-x86_64`
     * * `resolve:ssm:/aws/service/cloud9/amis/amazonlinux-2023-x86_64`
     * * `resolve:ssm:/aws/service/cloud9/amis/ubuntu-18.04-x86_64`
     * * `resolve:ssm:/aws/service/cloud9/amis/ubuntu-22.04-x86_64`
     */
    readonly imageId: pulumi.Output<string>;
    /**
     * The type of instance to connect to the environment, e.g., `t2.micro`.
     */
    readonly instanceType: pulumi.Output<string>;
    /**
     * The name of the environment.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The ARN of the environment owner. This can be ARN of any AWS IAM principal. Defaults to the environment's creator.
     */
    readonly ownerArn: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The ID of the subnet in Amazon VPC that AWS Cloud9 will use to communicate with the Amazon EC2 instance.
     */
    readonly subnetId: pulumi.Output<string | undefined>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The type of the environment (e.g., `ssh` or `ec2`).
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a EnvironmentEC2 resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EnvironmentEC2Args, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EnvironmentEC2 resources.
 */
export interface EnvironmentEC2State {
    /**
     * The ARN of the environment.
     */
    arn?: pulumi.Input<string>;
    /**
     * The number of minutes until the running instance is shut down after the environment has last been used.
     */
    automaticStopTimeMinutes?: pulumi.Input<number>;
    /**
     * The connection type used for connecting to an Amazon EC2 environment. Valid values are `CONNECT_SSH` and `CONNECT_SSM`. For more information please refer [AWS documentation for Cloud9](https://docs.aws.amazon.com/cloud9/latest/user-guide/ec2-ssm.html).
     */
    connectionType?: pulumi.Input<string>;
    /**
     * The description of the environment.
     */
    description?: pulumi.Input<string>;
    /**
     * The identifier for the Amazon Machine Image (AMI) that's used to create the EC2 instance. Valid values are
     * * `amazonlinux-2-x86_64`
     * * `amazonlinux-2023-x86_64`
     * * `ubuntu-18.04-x86_64`
     * * `ubuntu-22.04-x86_64`
     * * `resolve:ssm:/aws/service/cloud9/amis/amazonlinux-2-x86_64`
     * * `resolve:ssm:/aws/service/cloud9/amis/amazonlinux-2023-x86_64`
     * * `resolve:ssm:/aws/service/cloud9/amis/ubuntu-18.04-x86_64`
     * * `resolve:ssm:/aws/service/cloud9/amis/ubuntu-22.04-x86_64`
     */
    imageId?: pulumi.Input<string>;
    /**
     * The type of instance to connect to the environment, e.g., `t2.micro`.
     */
    instanceType?: pulumi.Input<string>;
    /**
     * The name of the environment.
     */
    name?: pulumi.Input<string>;
    /**
     * The ARN of the environment owner. This can be ARN of any AWS IAM principal. Defaults to the environment's creator.
     */
    ownerArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the subnet in Amazon VPC that AWS Cloud9 will use to communicate with the Amazon EC2 instance.
     */
    subnetId?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The type of the environment (e.g., `ssh` or `ec2`).
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EnvironmentEC2 resource.
 */
export interface EnvironmentEC2Args {
    /**
     * The number of minutes until the running instance is shut down after the environment has last been used.
     */
    automaticStopTimeMinutes?: pulumi.Input<number>;
    /**
     * The connection type used for connecting to an Amazon EC2 environment. Valid values are `CONNECT_SSH` and `CONNECT_SSM`. For more information please refer [AWS documentation for Cloud9](https://docs.aws.amazon.com/cloud9/latest/user-guide/ec2-ssm.html).
     */
    connectionType?: pulumi.Input<string>;
    /**
     * The description of the environment.
     */
    description?: pulumi.Input<string>;
    /**
     * The identifier for the Amazon Machine Image (AMI) that's used to create the EC2 instance. Valid values are
     * * `amazonlinux-2-x86_64`
     * * `amazonlinux-2023-x86_64`
     * * `ubuntu-18.04-x86_64`
     * * `ubuntu-22.04-x86_64`
     * * `resolve:ssm:/aws/service/cloud9/amis/amazonlinux-2-x86_64`
     * * `resolve:ssm:/aws/service/cloud9/amis/amazonlinux-2023-x86_64`
     * * `resolve:ssm:/aws/service/cloud9/amis/ubuntu-18.04-x86_64`
     * * `resolve:ssm:/aws/service/cloud9/amis/ubuntu-22.04-x86_64`
     */
    imageId: pulumi.Input<string>;
    /**
     * The type of instance to connect to the environment, e.g., `t2.micro`.
     */
    instanceType: pulumi.Input<string>;
    /**
     * The name of the environment.
     */
    name?: pulumi.Input<string>;
    /**
     * The ARN of the environment owner. This can be ARN of any AWS IAM principal. Defaults to the environment's creator.
     */
    ownerArn?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The ID of the subnet in Amazon VPC that AWS Cloud9 will use to communicate with the Amazon EC2 instance.
     */
    subnetId?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
