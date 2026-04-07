import * as pulumi from "@pulumi/pulumi";
/**
 * Provides an Elastic MapReduce Studio.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.emr.Studio("example", {
 *     authMode: "SSO",
 *     defaultS3Location: `s3://${test.bucket}/test`,
 *     engineSecurityGroupId: testAwsSecurityGroup.id,
 *     name: "example",
 *     serviceRole: testAwsIamRole.arn,
 *     subnetIds: [testAwsSubnet.id],
 *     userRole: testAwsIamRole.arn,
 *     vpcId: testAwsVpc.id,
 *     workspaceSecurityGroupId: testAwsSecurityGroup.id,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import EMR studios using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:emr/studio:Studio studio es-123456ABCDEF
 * ```
 */
export declare class Studio extends pulumi.CustomResource {
    /**
     * Get an existing Studio resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: StudioState, opts?: pulumi.CustomResourceOptions): Studio;
    /**
     * Returns true if the given object is an instance of Studio.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Studio;
    /**
     * ARN of the studio.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Specifies whether the Studio authenticates users using IAM or Amazon Web Services SSO. Valid values are `SSO` or `IAM`.
     */
    readonly authMode: pulumi.Output<string>;
    /**
     * The Amazon S3 location to back up Amazon EMR Studio Workspaces and notebook files.
     */
    readonly defaultS3Location: pulumi.Output<string>;
    /**
     * A detailed description of the Amazon EMR Studio.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The AWS KMS key identifier (ARN) used to encrypt Amazon EMR Studio workspace and notebook files when backed up to Amazon S3.
     */
    readonly encryptionKeyArn: pulumi.Output<string | undefined>;
    /**
     * The ID of the Amazon EMR Studio Engine security group. The Engine security group allows inbound network traffic from the Workspace security group, and it must be in the same VPC specified by `vpcId`.
     */
    readonly engineSecurityGroupId: pulumi.Output<string>;
    /**
     * The authentication endpoint of your identity provider (IdP). Specify this value when you use IAM authentication and want to let federated users log in to a Studio with the Studio URL and credentials from your IdP. Amazon EMR Studio redirects users to this endpoint to enter credentials.
     */
    readonly idpAuthUrl: pulumi.Output<string | undefined>;
    /**
     * The name that your identity provider (IdP) uses for its RelayState parameter. For example, RelayState or TargetSource. Specify this value when you use IAM authentication and want to let federated users log in to a Studio using the Studio URL. The RelayState parameter differs by IdP.
     */
    readonly idpRelayStateParameterName: pulumi.Output<string | undefined>;
    /**
     * A descriptive name for the Amazon EMR Studio.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The IAM role that the Amazon EMR Studio assumes. The service role provides a way for Amazon EMR Studio to interoperate with other Amazon Web Services services.
     */
    readonly serviceRole: pulumi.Output<string>;
    /**
     * A list of subnet IDs to associate with the Amazon EMR Studio. A Studio can have a maximum of 5 subnets. The subnets must belong to the VPC specified by `vpcId`. Studio users can create a Workspace in any of the specified subnets.
     */
    readonly subnetIds: pulumi.Output<string[]>;
    /**
     * list of tags to apply to the EMR Cluster. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * The unique access URL of the Amazon EMR Studio.
     */
    readonly url: pulumi.Output<string>;
    /**
     * The IAM user role that users and groups assume when logged in to an Amazon EMR Studio. Only specify a User Role when you use Amazon Web Services SSO authentication. The permissions attached to the User Role can be scoped down for each user or group using session policies.
     */
    readonly userRole: pulumi.Output<string | undefined>;
    /**
     * The ID of the Amazon Virtual Private Cloud (Amazon VPC) to associate with the Studio.
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * The ID of the Amazon EMR Studio Workspace security group. The Workspace security group allows outbound network traffic to resources in the Engine security group, and it must be in the same VPC specified by `vpcId`.
     *
     * The following arguments are optional:
     */
    readonly workspaceSecurityGroupId: pulumi.Output<string>;
    /**
     * Create a Studio resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: StudioArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Studio resources.
 */
export interface StudioState {
    /**
     * ARN of the studio.
     */
    arn?: pulumi.Input<string>;
    /**
     * Specifies whether the Studio authenticates users using IAM or Amazon Web Services SSO. Valid values are `SSO` or `IAM`.
     */
    authMode?: pulumi.Input<string>;
    /**
     * The Amazon S3 location to back up Amazon EMR Studio Workspaces and notebook files.
     */
    defaultS3Location?: pulumi.Input<string>;
    /**
     * A detailed description of the Amazon EMR Studio.
     */
    description?: pulumi.Input<string>;
    /**
     * The AWS KMS key identifier (ARN) used to encrypt Amazon EMR Studio workspace and notebook files when backed up to Amazon S3.
     */
    encryptionKeyArn?: pulumi.Input<string>;
    /**
     * The ID of the Amazon EMR Studio Engine security group. The Engine security group allows inbound network traffic from the Workspace security group, and it must be in the same VPC specified by `vpcId`.
     */
    engineSecurityGroupId?: pulumi.Input<string>;
    /**
     * The authentication endpoint of your identity provider (IdP). Specify this value when you use IAM authentication and want to let federated users log in to a Studio with the Studio URL and credentials from your IdP. Amazon EMR Studio redirects users to this endpoint to enter credentials.
     */
    idpAuthUrl?: pulumi.Input<string>;
    /**
     * The name that your identity provider (IdP) uses for its RelayState parameter. For example, RelayState or TargetSource. Specify this value when you use IAM authentication and want to let federated users log in to a Studio using the Studio URL. The RelayState parameter differs by IdP.
     */
    idpRelayStateParameterName?: pulumi.Input<string>;
    /**
     * A descriptive name for the Amazon EMR Studio.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IAM role that the Amazon EMR Studio assumes. The service role provides a way for Amazon EMR Studio to interoperate with other Amazon Web Services services.
     */
    serviceRole?: pulumi.Input<string>;
    /**
     * A list of subnet IDs to associate with the Amazon EMR Studio. A Studio can have a maximum of 5 subnets. The subnets must belong to the VPC specified by `vpcId`. Studio users can create a Workspace in any of the specified subnets.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * list of tags to apply to the EMR Cluster. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The unique access URL of the Amazon EMR Studio.
     */
    url?: pulumi.Input<string>;
    /**
     * The IAM user role that users and groups assume when logged in to an Amazon EMR Studio. Only specify a User Role when you use Amazon Web Services SSO authentication. The permissions attached to the User Role can be scoped down for each user or group using session policies.
     */
    userRole?: pulumi.Input<string>;
    /**
     * The ID of the Amazon Virtual Private Cloud (Amazon VPC) to associate with the Studio.
     */
    vpcId?: pulumi.Input<string>;
    /**
     * The ID of the Amazon EMR Studio Workspace security group. The Workspace security group allows outbound network traffic to resources in the Engine security group, and it must be in the same VPC specified by `vpcId`.
     *
     * The following arguments are optional:
     */
    workspaceSecurityGroupId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Studio resource.
 */
export interface StudioArgs {
    /**
     * Specifies whether the Studio authenticates users using IAM or Amazon Web Services SSO. Valid values are `SSO` or `IAM`.
     */
    authMode: pulumi.Input<string>;
    /**
     * The Amazon S3 location to back up Amazon EMR Studio Workspaces and notebook files.
     */
    defaultS3Location: pulumi.Input<string>;
    /**
     * A detailed description of the Amazon EMR Studio.
     */
    description?: pulumi.Input<string>;
    /**
     * The AWS KMS key identifier (ARN) used to encrypt Amazon EMR Studio workspace and notebook files when backed up to Amazon S3.
     */
    encryptionKeyArn?: pulumi.Input<string>;
    /**
     * The ID of the Amazon EMR Studio Engine security group. The Engine security group allows inbound network traffic from the Workspace security group, and it must be in the same VPC specified by `vpcId`.
     */
    engineSecurityGroupId: pulumi.Input<string>;
    /**
     * The authentication endpoint of your identity provider (IdP). Specify this value when you use IAM authentication and want to let federated users log in to a Studio with the Studio URL and credentials from your IdP. Amazon EMR Studio redirects users to this endpoint to enter credentials.
     */
    idpAuthUrl?: pulumi.Input<string>;
    /**
     * The name that your identity provider (IdP) uses for its RelayState parameter. For example, RelayState or TargetSource. Specify this value when you use IAM authentication and want to let federated users log in to a Studio using the Studio URL. The RelayState parameter differs by IdP.
     */
    idpRelayStateParameterName?: pulumi.Input<string>;
    /**
     * A descriptive name for the Amazon EMR Studio.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The IAM role that the Amazon EMR Studio assumes. The service role provides a way for Amazon EMR Studio to interoperate with other Amazon Web Services services.
     */
    serviceRole: pulumi.Input<string>;
    /**
     * A list of subnet IDs to associate with the Amazon EMR Studio. A Studio can have a maximum of 5 subnets. The subnets must belong to the VPC specified by `vpcId`. Studio users can create a Workspace in any of the specified subnets.
     */
    subnetIds: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * list of tags to apply to the EMR Cluster. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The IAM user role that users and groups assume when logged in to an Amazon EMR Studio. Only specify a User Role when you use Amazon Web Services SSO authentication. The permissions attached to the User Role can be scoped down for each user or group using session policies.
     */
    userRole?: pulumi.Input<string>;
    /**
     * The ID of the Amazon Virtual Private Cloud (Amazon VPC) to associate with the Studio.
     */
    vpcId: pulumi.Input<string>;
    /**
     * The ID of the Amazon EMR Studio Workspace security group. The Workspace security group allows outbound network traffic to resources in the Engine security group, and it must be in the same VPC specified by `vpcId`.
     *
     * The following arguments are optional:
     */
    workspaceSecurityGroupId: pulumi.Input<string>;
}
