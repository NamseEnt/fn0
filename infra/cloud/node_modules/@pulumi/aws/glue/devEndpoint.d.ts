import * as pulumi from "@pulumi/pulumi";
/**
 * Provides a Glue Development Endpoint resource.
 *
 * ## Example Usage
 *
 * Basic usage:
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.iam.getPolicyDocument({
 *     statements: [{
 *         actions: ["sts:AssumeRole"],
 *         principals: [{
 *             type: "Service",
 *             identifiers: ["glue.amazonaws.com"],
 *         }],
 *     }],
 * });
 * const exampleRole = new aws.iam.Role("example", {
 *     name: "AWSGlueServiceRole-foo",
 *     assumeRolePolicy: example.then(example => example.json),
 * });
 * const exampleDevEndpoint = new aws.glue.DevEndpoint("example", {
 *     name: "foo",
 *     roleArn: exampleRole.arn,
 * });
 * const example_AWSGlueServiceRole = new aws.iam.RolePolicyAttachment("example-AWSGlueServiceRole", {
 *     policyArn: "arn:aws:iam::aws:policy/service-role/AWSGlueServiceRole",
 *     role: exampleRole.name,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import a Glue Development Endpoint using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:glue/devEndpoint:DevEndpoint example foo
 * ```
 */
export declare class DevEndpoint extends pulumi.CustomResource {
    /**
     * Get an existing DevEndpoint resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DevEndpointState, opts?: pulumi.CustomResourceOptions): DevEndpoint;
    /**
     * Returns true if the given object is an instance of DevEndpoint.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DevEndpoint;
    /**
     * A map of arguments used to configure the endpoint.
     */
    readonly arguments: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * The ARN of the endpoint.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The AWS availability zone where this endpoint is located.
     */
    readonly availabilityZone: pulumi.Output<string>;
    /**
     * Path to one or more Java Jars in an S3 bucket that should be loaded in this endpoint.
     */
    readonly extraJarsS3Path: pulumi.Output<string | undefined>;
    /**
     * Path(s) to one or more Python libraries in an S3 bucket that should be loaded in this endpoint. Multiple values must be complete paths separated by a comma.
     */
    readonly extraPythonLibsS3Path: pulumi.Output<string | undefined>;
    /**
     * The reason for a current failure in this endpoint.
     */
    readonly failureReason: pulumi.Output<string>;
    /**
     * Specifies the versions of Python and Apache Spark to use. Defaults to AWS Glue version 0.9.
     */
    readonly glueVersion: pulumi.Output<string | undefined>;
    /**
     * The name of this endpoint. It must be unique in your account.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The number of AWS Glue Data Processing Units (DPUs) to allocate to this endpoint. Conflicts with `workerType`.
     */
    readonly numberOfNodes: pulumi.Output<number | undefined>;
    /**
     * The number of workers of a defined worker type that are allocated to this endpoint. This field is available only when you choose worker type G.1X or G.2X.
     */
    readonly numberOfWorkers: pulumi.Output<number | undefined>;
    /**
     * A private IP address to access the endpoint within a VPC, if this endpoint is created within one.
     */
    readonly privateAddress: pulumi.Output<string>;
    /**
     * The public IP address used by this endpoint. The PublicAddress field is present only when you create a non-VPC endpoint.
     */
    readonly publicAddress: pulumi.Output<string>;
    /**
     * The public key to be used by this endpoint for authentication.
     */
    readonly publicKey: pulumi.Output<string | undefined>;
    /**
     * A list of public keys to be used by this endpoint for authentication.
     */
    readonly publicKeys: pulumi.Output<string[] | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The IAM role for this endpoint.
     */
    readonly roleArn: pulumi.Output<string>;
    /**
     * The name of the Security Configuration structure to be used with this endpoint.
     */
    readonly securityConfiguration: pulumi.Output<string | undefined>;
    /**
     * Security group IDs for the security groups to be used by this endpoint.
     */
    readonly securityGroupIds: pulumi.Output<string[] | undefined>;
    /**
     * The current status of this endpoint.
     */
    readonly status: pulumi.Output<string>;
    /**
     * The subnet ID for the new endpoint to use.
     */
    readonly subnetId: pulumi.Output<string | undefined>;
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
    /**
     * he ID of the VPC used by this endpoint.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * The type of predefined worker that is allocated to this endpoint. Accepts a value of Standard, G.1X, or G.2X.
     */
    readonly workerType: pulumi.Output<string | undefined>;
    /**
     * The YARN endpoint address used by this endpoint.
     */
    readonly yarnEndpointAddress: pulumi.Output<string>;
    /**
     * The Apache Zeppelin port for the remote Apache Spark interpreter.
     */
    readonly zeppelinRemoteSparkInterpreterPort: pulumi.Output<number>;
    /**
     * Create a DevEndpoint resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DevEndpointArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DevEndpoint resources.
 */
export interface DevEndpointState {
    /**
     * A map of arguments used to configure the endpoint.
     */
    arguments?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ARN of the endpoint.
     */
    arn?: pulumi.Input<string>;
    /**
     * The AWS availability zone where this endpoint is located.
     */
    availabilityZone?: pulumi.Input<string>;
    /**
     * Path to one or more Java Jars in an S3 bucket that should be loaded in this endpoint.
     */
    extraJarsS3Path?: pulumi.Input<string>;
    /**
     * Path(s) to one or more Python libraries in an S3 bucket that should be loaded in this endpoint. Multiple values must be complete paths separated by a comma.
     */
    extraPythonLibsS3Path?: pulumi.Input<string>;
    /**
     * The reason for a current failure in this endpoint.
     */
    failureReason?: pulumi.Input<string>;
    /**
     * Specifies the versions of Python and Apache Spark to use. Defaults to AWS Glue version 0.9.
     */
    glueVersion?: pulumi.Input<string>;
    /**
     * The name of this endpoint. It must be unique in your account.
     */
    name?: pulumi.Input<string>;
    /**
     * The number of AWS Glue Data Processing Units (DPUs) to allocate to this endpoint. Conflicts with `workerType`.
     */
    numberOfNodes?: pulumi.Input<number>;
    /**
     * The number of workers of a defined worker type that are allocated to this endpoint. This field is available only when you choose worker type G.1X or G.2X.
     */
    numberOfWorkers?: pulumi.Input<number>;
    /**
     * A private IP address to access the endpoint within a VPC, if this endpoint is created within one.
     */
    privateAddress?: pulumi.Input<string>;
    /**
     * The public IP address used by this endpoint. The PublicAddress field is present only when you create a non-VPC endpoint.
     */
    publicAddress?: pulumi.Input<string>;
    /**
     * The public key to be used by this endpoint for authentication.
     */
    publicKey?: pulumi.Input<string>;
    /**
     * A list of public keys to be used by this endpoint for authentication.
     */
    publicKeys?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IAM role for this endpoint.
     */
    roleArn?: pulumi.Input<string>;
    /**
     * The name of the Security Configuration structure to be used with this endpoint.
     */
    securityConfiguration?: pulumi.Input<string>;
    /**
     * Security group IDs for the security groups to be used by this endpoint.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The current status of this endpoint.
     */
    status?: pulumi.Input<string>;
    /**
     * The subnet ID for the new endpoint to use.
     */
    subnetId?: pulumi.Input<string>;
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
    /**
     * he ID of the VPC used by this endpoint.
     */
    vpcId?: pulumi.Input<string>;
    /**
     * The type of predefined worker that is allocated to this endpoint. Accepts a value of Standard, G.1X, or G.2X.
     */
    workerType?: pulumi.Input<string>;
    /**
     * The YARN endpoint address used by this endpoint.
     */
    yarnEndpointAddress?: pulumi.Input<string>;
    /**
     * The Apache Zeppelin port for the remote Apache Spark interpreter.
     */
    zeppelinRemoteSparkInterpreterPort?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a DevEndpoint resource.
 */
export interface DevEndpointArgs {
    /**
     * A map of arguments used to configure the endpoint.
     */
    arguments?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Path to one or more Java Jars in an S3 bucket that should be loaded in this endpoint.
     */
    extraJarsS3Path?: pulumi.Input<string>;
    /**
     * Path(s) to one or more Python libraries in an S3 bucket that should be loaded in this endpoint. Multiple values must be complete paths separated by a comma.
     */
    extraPythonLibsS3Path?: pulumi.Input<string>;
    /**
     * Specifies the versions of Python and Apache Spark to use. Defaults to AWS Glue version 0.9.
     */
    glueVersion?: pulumi.Input<string>;
    /**
     * The name of this endpoint. It must be unique in your account.
     */
    name?: pulumi.Input<string>;
    /**
     * The number of AWS Glue Data Processing Units (DPUs) to allocate to this endpoint. Conflicts with `workerType`.
     */
    numberOfNodes?: pulumi.Input<number>;
    /**
     * The number of workers of a defined worker type that are allocated to this endpoint. This field is available only when you choose worker type G.1X or G.2X.
     */
    numberOfWorkers?: pulumi.Input<number>;
    /**
     * The public key to be used by this endpoint for authentication.
     */
    publicKey?: pulumi.Input<string>;
    /**
     * A list of public keys to be used by this endpoint for authentication.
     */
    publicKeys?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IAM role for this endpoint.
     */
    roleArn: pulumi.Input<string>;
    /**
     * The name of the Security Configuration structure to be used with this endpoint.
     */
    securityConfiguration?: pulumi.Input<string>;
    /**
     * Security group IDs for the security groups to be used by this endpoint.
     */
    securityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The subnet ID for the new endpoint to use.
     */
    subnetId?: pulumi.Input<string>;
    /**
     * Key-value map of resource tags. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The type of predefined worker that is allocated to this endpoint. Accepts a value of Standard, G.1X, or G.2X.
     */
    workerType?: pulumi.Input<string>;
}
