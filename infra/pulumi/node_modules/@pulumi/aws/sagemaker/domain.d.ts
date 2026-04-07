import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a SageMaker AI Domain resource.
 *
 * ## Example Usage
 *
 * ### Basic usage
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
 *             identifiers: ["sagemaker.amazonaws.com"],
 *         }],
 *     }],
 * });
 * const exampleRole = new aws.iam.Role("example", {
 *     name: "example",
 *     path: "/",
 *     assumeRolePolicy: example.then(example => example.json),
 * });
 * const exampleDomain = new aws.sagemaker.Domain("example", {
 *     domainName: "example",
 *     authMode: "IAM",
 *     vpcId: exampleAwsVpc.id,
 *     subnetIds: [exampleAwsSubnet.id],
 *     defaultUserSettings: {
 *         executionRole: exampleRole.arn,
 *     },
 * });
 * ```
 *
 * ### Using Custom Images
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.sagemaker.Image("example", {
 *     imageName: "example",
 *     roleArn: exampleAwsIamRole.arn,
 * });
 * const exampleAppImageConfig = new aws.sagemaker.AppImageConfig("example", {
 *     appImageConfigName: "example",
 *     kernelGatewayImageConfig: {
 *         kernelSpecs: [{
 *             name: "example",
 *         }],
 *     },
 * });
 * const exampleImageVersion = new aws.sagemaker.ImageVersion("example", {
 *     imageName: example.id,
 *     baseImage: "base-image",
 * });
 * const exampleDomain = new aws.sagemaker.Domain("example", {
 *     domainName: "example",
 *     authMode: "IAM",
 *     vpcId: exampleAwsVpc.id,
 *     subnetIds: [exampleAwsSubnet.id],
 *     defaultUserSettings: {
 *         executionRole: exampleAwsIamRole.arn,
 *         kernelGatewayAppSettings: {
 *             customImages: [{
 *                 appImageConfigName: exampleAppImageConfig.appImageConfigName,
 *                 imageName: exampleImageVersion.imageName,
 *             }],
 *         },
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import SageMaker AI Domains using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:sagemaker/domain:Domain test_domain d-8jgsjtilstu8
 * ```
 */
export declare class Domain extends pulumi.CustomResource {
    /**
     * Get an existing Domain resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DomainState, opts?: pulumi.CustomResourceOptions): Domain;
    /**
     * Returns true if the given object is an instance of Domain.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Domain;
    /**
     * Specifies the VPC used for non-EFS traffic. The default value is `PublicInternetOnly`. Valid values are `PublicInternetOnly` and `VpcOnly`.
     */
    readonly appNetworkAccessType: pulumi.Output<string | undefined>;
    /**
     * The entity that creates and manages the required security groups for inter-app communication in `VPCOnly` mode. Valid values are `Service` and `Customer`.
     */
    readonly appSecurityGroupManagement: pulumi.Output<string | undefined>;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Domain.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The mode of authentication that members use to access the domain. Valid values are `IAM` and `SSO`.
     */
    readonly authMode: pulumi.Output<string>;
    /**
     * The default space settings. See `defaultSpaceSettings` Block below.
     */
    readonly defaultSpaceSettings: pulumi.Output<outputs.sagemaker.DomainDefaultSpaceSettings | undefined>;
    /**
     * The default user settings. See `defaultUserSettings` Block below.
     */
    readonly defaultUserSettings: pulumi.Output<outputs.sagemaker.DomainDefaultUserSettings>;
    /**
     * The domain name.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * The domain settings. See `domainSettings` Block below.
     */
    readonly domainSettings: pulumi.Output<outputs.sagemaker.DomainDomainSettings | undefined>;
    /**
     * The ID of the Amazon Elastic File System (EFS) managed by this Domain.
     */
    readonly homeEfsFileSystemId: pulumi.Output<string>;
    /**
     * The AWS KMS customer managed CMK used to encrypt the EFS volume attached to the domain.
     */
    readonly kmsKeyId: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The retention policy for this domain, which specifies whether resources will be retained after the Domain is deleted. By default, all resources are retained. See `retentionPolicy` Block below.
     */
    readonly retentionPolicy: pulumi.Output<outputs.sagemaker.DomainRetentionPolicy | undefined>;
    /**
     * The ID of the security group that authorizes traffic between the RSessionGateway apps and the RStudioServerPro app.
     */
    readonly securityGroupIdForDomainBoundary: pulumi.Output<string>;
    /**
     * The ARN of the application managed by SageMaker AI in IAM Identity Center. This value is only returned for domains created after September 19, 2023.
     */
    readonly singleSignOnApplicationArn: pulumi.Output<string>;
    /**
     * The SSO managed application instance ID.
     */
    readonly singleSignOnManagedApplicationInstanceId: pulumi.Output<string>;
    /**
     * The VPC subnets that Studio uses for communication.
     */
    readonly subnetIds: pulumi.Output<string[]>;
    /**
     * Indicates whether custom tag propagation is supported for the domain. Defaults to `DISABLED`. Valid values are: `ENABLED` and `DISABLED`.
     */
    readonly tagPropagation: pulumi.Output<string | undefined>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The domain's URL.
     */
    readonly url: pulumi.Output<string>;
    /**
     * The ID of the Amazon Virtual Private Cloud (VPC) that Studio uses for communication.
     *
     * The following arguments are optional:
     */
    readonly vpcId: pulumi.Output<string>;
    /**
     * Create a Domain resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DomainArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Domain resources.
 */
export interface DomainState {
    /**
     * Specifies the VPC used for non-EFS traffic. The default value is `PublicInternetOnly`. Valid values are `PublicInternetOnly` and `VpcOnly`.
     */
    appNetworkAccessType?: pulumi.Input<string>;
    /**
     * The entity that creates and manages the required security groups for inter-app communication in `VPCOnly` mode. Valid values are `Service` and `Customer`.
     */
    appSecurityGroupManagement?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) assigned by AWS to this Domain.
     */
    arn?: pulumi.Input<string>;
    /**
     * The mode of authentication that members use to access the domain. Valid values are `IAM` and `SSO`.
     */
    authMode?: pulumi.Input<string>;
    /**
     * The default space settings. See `defaultSpaceSettings` Block below.
     */
    defaultSpaceSettings?: pulumi.Input<inputs.sagemaker.DomainDefaultSpaceSettings>;
    /**
     * The default user settings. See `defaultUserSettings` Block below.
     */
    defaultUserSettings?: pulumi.Input<inputs.sagemaker.DomainDefaultUserSettings>;
    /**
     * The domain name.
     */
    domainName?: pulumi.Input<string>;
    /**
     * The domain settings. See `domainSettings` Block below.
     */
    domainSettings?: pulumi.Input<inputs.sagemaker.DomainDomainSettings>;
    /**
     * The ID of the Amazon Elastic File System (EFS) managed by this Domain.
     */
    homeEfsFileSystemId?: pulumi.Input<string>;
    /**
     * The AWS KMS customer managed CMK used to encrypt the EFS volume attached to the domain.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The retention policy for this domain, which specifies whether resources will be retained after the Domain is deleted. By default, all resources are retained. See `retentionPolicy` Block below.
     */
    retentionPolicy?: pulumi.Input<inputs.sagemaker.DomainRetentionPolicy>;
    /**
     * The ID of the security group that authorizes traffic between the RSessionGateway apps and the RStudioServerPro app.
     */
    securityGroupIdForDomainBoundary?: pulumi.Input<string>;
    /**
     * The ARN of the application managed by SageMaker AI in IAM Identity Center. This value is only returned for domains created after September 19, 2023.
     */
    singleSignOnApplicationArn?: pulumi.Input<string>;
    /**
     * The SSO managed application instance ID.
     */
    singleSignOnManagedApplicationInstanceId?: pulumi.Input<string>;
    /**
     * The VPC subnets that Studio uses for communication.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Indicates whether custom tag propagation is supported for the domain. Defaults to `DISABLED`. Valid values are: `ENABLED` and `DISABLED`.
     */
    tagPropagation?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The domain's URL.
     */
    url?: pulumi.Input<string>;
    /**
     * The ID of the Amazon Virtual Private Cloud (VPC) that Studio uses for communication.
     *
     * The following arguments are optional:
     */
    vpcId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a Domain resource.
 */
export interface DomainArgs {
    /**
     * Specifies the VPC used for non-EFS traffic. The default value is `PublicInternetOnly`. Valid values are `PublicInternetOnly` and `VpcOnly`.
     */
    appNetworkAccessType?: pulumi.Input<string>;
    /**
     * The entity that creates and manages the required security groups for inter-app communication in `VPCOnly` mode. Valid values are `Service` and `Customer`.
     */
    appSecurityGroupManagement?: pulumi.Input<string>;
    /**
     * The mode of authentication that members use to access the domain. Valid values are `IAM` and `SSO`.
     */
    authMode: pulumi.Input<string>;
    /**
     * The default space settings. See `defaultSpaceSettings` Block below.
     */
    defaultSpaceSettings?: pulumi.Input<inputs.sagemaker.DomainDefaultSpaceSettings>;
    /**
     * The default user settings. See `defaultUserSettings` Block below.
     */
    defaultUserSettings: pulumi.Input<inputs.sagemaker.DomainDefaultUserSettings>;
    /**
     * The domain name.
     */
    domainName: pulumi.Input<string>;
    /**
     * The domain settings. See `domainSettings` Block below.
     */
    domainSettings?: pulumi.Input<inputs.sagemaker.DomainDomainSettings>;
    /**
     * The AWS KMS customer managed CMK used to encrypt the EFS volume attached to the domain.
     */
    kmsKeyId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The retention policy for this domain, which specifies whether resources will be retained after the Domain is deleted. By default, all resources are retained. See `retentionPolicy` Block below.
     */
    retentionPolicy?: pulumi.Input<inputs.sagemaker.DomainRetentionPolicy>;
    /**
     * The VPC subnets that Studio uses for communication.
     */
    subnetIds: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Indicates whether custom tag propagation is supported for the domain. Defaults to `DISABLED`. Valid values are: `ENABLED` and `DISABLED`.
     */
    tagPropagation?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. .If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The ID of the Amazon Virtual Private Cloud (VPC) that Studio uses for communication.
     *
     * The following arguments are optional:
     */
    vpcId: pulumi.Input<string>;
}
