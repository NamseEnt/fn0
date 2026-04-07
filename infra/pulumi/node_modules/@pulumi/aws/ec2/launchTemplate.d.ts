import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides an EC2 launch template resource. Can be used to create instances or auto scaling groups.
 *
 * ## Import
 *
 * Using `pulumi import`, import Launch Templates using the `id`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/launchTemplate:LaunchTemplate web lt-12345678
 * ```
 */
export declare class LaunchTemplate extends pulumi.CustomResource {
    /**
     * Get an existing LaunchTemplate resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: LaunchTemplateState, opts?: pulumi.CustomResourceOptions): LaunchTemplate;
    /**
     * Returns true if the given object is an instance of LaunchTemplate.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is LaunchTemplate;
    /**
     * Amazon Resource Name (ARN) of the launch template.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Specify volumes to attach to the instance besides the volumes specified by the AMI.
     * See Block Devices below for details.
     */
    readonly blockDeviceMappings: pulumi.Output<outputs.ec2.LaunchTemplateBlockDeviceMapping[] | undefined>;
    /**
     * Targeting for EC2 capacity reservations. See Capacity Reservation Specification below for more details.
     */
    readonly capacityReservationSpecification: pulumi.Output<outputs.ec2.LaunchTemplateCapacityReservationSpecification | undefined>;
    /**
     * The CPU options for the instance. See CPU Options below for more details.
     */
    readonly cpuOptions: pulumi.Output<outputs.ec2.LaunchTemplateCpuOptions | undefined>;
    /**
     * Customize the credit specification of the instance. See Credit
     * Specification below for more details.
     */
    readonly creditSpecification: pulumi.Output<outputs.ec2.LaunchTemplateCreditSpecification | undefined>;
    /**
     * Default Version of the launch template.
     */
    readonly defaultVersion: pulumi.Output<number>;
    /**
     * Description of the launch template.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * If true, enables [EC2 Instance Stop Protection](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-stop-protection.html).
     */
    readonly disableApiStop: pulumi.Output<boolean | undefined>;
    /**
     * If `true`, enables [EC2 Instance
     * Termination Protection](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_ChangingDisableAPITermination.html)
     */
    readonly disableApiTermination: pulumi.Output<boolean | undefined>;
    /**
     * If `true`, the launched EC2 instance will be EBS-optimized.
     */
    readonly ebsOptimized: pulumi.Output<string | undefined>;
    /**
     * Enable Nitro Enclaves on launched instances. See Enclave Options below for more details.
     */
    readonly enclaveOptions: pulumi.Output<outputs.ec2.LaunchTemplateEnclaveOptions | undefined>;
    /**
     * The hibernation options for the instance. See Hibernation Options below for more details.
     */
    readonly hibernationOptions: pulumi.Output<outputs.ec2.LaunchTemplateHibernationOptions | undefined>;
    /**
     * The IAM Instance Profile to launch the instance with. See Instance Profile
     * below for more details.
     */
    readonly iamInstanceProfile: pulumi.Output<outputs.ec2.LaunchTemplateIamInstanceProfile | undefined>;
    /**
     * The AMI from which to launch the instance or use a Systems Manager parameter convention e.g. `resolve:ssm:parameter-name`. See [docs](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/create-launch-template.html#use-an-ssm-parameter-instead-of-an-ami-id) for more details.
     */
    readonly imageId: pulumi.Output<string | undefined>;
    /**
     * Shutdown behavior for the instance. Can be `stop` or `terminate`.
     * (Default: `stop`).
     */
    readonly instanceInitiatedShutdownBehavior: pulumi.Output<string | undefined>;
    /**
     * The market (purchasing) option for the instance. See Market Options
     * below for details.
     */
    readonly instanceMarketOptions: pulumi.Output<outputs.ec2.LaunchTemplateInstanceMarketOptions | undefined>;
    /**
     * The attribute requirements for the type of instance. If present then `instanceType` cannot be present.
     */
    readonly instanceRequirements: pulumi.Output<outputs.ec2.LaunchTemplateInstanceRequirements | undefined>;
    /**
     * The type of the instance. If present then `instanceRequirements` cannot be present.
     */
    readonly instanceType: pulumi.Output<string | undefined>;
    /**
     * The kernel ID.
     */
    readonly kernelId: pulumi.Output<string | undefined>;
    /**
     * The key name to use for the instance.
     */
    readonly keyName: pulumi.Output<string | undefined>;
    /**
     * The latest version of the launch template.
     */
    readonly latestVersion: pulumi.Output<number>;
    /**
     * A list of license specifications to associate with. See License Specification below for more details.
     */
    readonly licenseSpecifications: pulumi.Output<outputs.ec2.LaunchTemplateLicenseSpecification[] | undefined>;
    /**
     * The maintenance options for the instance. See Maintenance Options below for more details.
     */
    readonly maintenanceOptions: pulumi.Output<outputs.ec2.LaunchTemplateMaintenanceOptions | undefined>;
    /**
     * Customize the metadata options for the instance. See Metadata Options below for more details.
     */
    readonly metadataOptions: pulumi.Output<outputs.ec2.LaunchTemplateMetadataOptions>;
    /**
     * The monitoring option for the instance. See Monitoring below for more details.
     */
    readonly monitoring: pulumi.Output<outputs.ec2.LaunchTemplateMonitoring | undefined>;
    /**
     * The name of the launch template. If you leave this blank, the provider will auto-generate a unique name.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    readonly namePrefix: pulumi.Output<string>;
    /**
     * Customize network interfaces to be attached at instance boot time. See Network
     * Interfaces below for more details.
     */
    readonly networkInterfaces: pulumi.Output<outputs.ec2.LaunchTemplateNetworkInterface[] | undefined>;
    /**
     * The placement of the instance. See Placement below for more details.
     */
    readonly placement: pulumi.Output<outputs.ec2.LaunchTemplatePlacement | undefined>;
    /**
     * The options for the instance hostname. The default values are inherited from the subnet. See Private DNS Name Options below for more details.
     */
    readonly privateDnsNameOptions: pulumi.Output<outputs.ec2.LaunchTemplatePrivateDnsNameOptions | undefined>;
    /**
     * The ID of the RAM disk.
     */
    readonly ramDiskId: pulumi.Output<string | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A list of security group names to associate with. If you are creating Instances in a VPC, use
     * `vpcSecurityGroupIds` instead.
     */
    readonly securityGroupNames: pulumi.Output<string[] | undefined>;
    /**
     * The tags to apply to the resources during launch. See Tag Specifications below for more details. Default tags are currently not propagated to ASG created resources so you may wish to inject your default tags into this variable against the relevant child resource types created.
     */
    readonly tagSpecifications: pulumi.Output<outputs.ec2.LaunchTemplateTagSpecification[] | undefined>;
    /**
     * A map of tags to assign to the launch template. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Whether to update Default Version each update. Conflicts with `defaultVersion`.
     */
    readonly updateDefaultVersion: pulumi.Output<boolean | undefined>;
    /**
     * The base64-encoded user data to provide when launching the instance.
     */
    readonly userData: pulumi.Output<string | undefined>;
    /**
     * A list of security group IDs to associate with. Conflicts with `network_interfaces.security_groups`
     */
    readonly vpcSecurityGroupIds: pulumi.Output<string[] | undefined>;
    /**
     * Create a LaunchTemplate resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: LaunchTemplateArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering LaunchTemplate resources.
 */
export interface LaunchTemplateState {
    /**
     * Amazon Resource Name (ARN) of the launch template.
     */
    arn?: pulumi.Input<string>;
    /**
     * Specify volumes to attach to the instance besides the volumes specified by the AMI.
     * See Block Devices below for details.
     */
    blockDeviceMappings?: pulumi.Input<pulumi.Input<inputs.ec2.LaunchTemplateBlockDeviceMapping>[]>;
    /**
     * Targeting for EC2 capacity reservations. See Capacity Reservation Specification below for more details.
     */
    capacityReservationSpecification?: pulumi.Input<inputs.ec2.LaunchTemplateCapacityReservationSpecification>;
    /**
     * The CPU options for the instance. See CPU Options below for more details.
     */
    cpuOptions?: pulumi.Input<inputs.ec2.LaunchTemplateCpuOptions>;
    /**
     * Customize the credit specification of the instance. See Credit
     * Specification below for more details.
     */
    creditSpecification?: pulumi.Input<inputs.ec2.LaunchTemplateCreditSpecification>;
    /**
     * Default Version of the launch template.
     */
    defaultVersion?: pulumi.Input<number>;
    /**
     * Description of the launch template.
     */
    description?: pulumi.Input<string>;
    /**
     * If true, enables [EC2 Instance Stop Protection](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-stop-protection.html).
     */
    disableApiStop?: pulumi.Input<boolean>;
    /**
     * If `true`, enables [EC2 Instance
     * Termination Protection](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_ChangingDisableAPITermination.html)
     */
    disableApiTermination?: pulumi.Input<boolean>;
    /**
     * If `true`, the launched EC2 instance will be EBS-optimized.
     */
    ebsOptimized?: pulumi.Input<string>;
    /**
     * Enable Nitro Enclaves on launched instances. See Enclave Options below for more details.
     */
    enclaveOptions?: pulumi.Input<inputs.ec2.LaunchTemplateEnclaveOptions>;
    /**
     * The hibernation options for the instance. See Hibernation Options below for more details.
     */
    hibernationOptions?: pulumi.Input<inputs.ec2.LaunchTemplateHibernationOptions>;
    /**
     * The IAM Instance Profile to launch the instance with. See Instance Profile
     * below for more details.
     */
    iamInstanceProfile?: pulumi.Input<inputs.ec2.LaunchTemplateIamInstanceProfile>;
    /**
     * The AMI from which to launch the instance or use a Systems Manager parameter convention e.g. `resolve:ssm:parameter-name`. See [docs](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/create-launch-template.html#use-an-ssm-parameter-instead-of-an-ami-id) for more details.
     */
    imageId?: pulumi.Input<string>;
    /**
     * Shutdown behavior for the instance. Can be `stop` or `terminate`.
     * (Default: `stop`).
     */
    instanceInitiatedShutdownBehavior?: pulumi.Input<string>;
    /**
     * The market (purchasing) option for the instance. See Market Options
     * below for details.
     */
    instanceMarketOptions?: pulumi.Input<inputs.ec2.LaunchTemplateInstanceMarketOptions>;
    /**
     * The attribute requirements for the type of instance. If present then `instanceType` cannot be present.
     */
    instanceRequirements?: pulumi.Input<inputs.ec2.LaunchTemplateInstanceRequirements>;
    /**
     * The type of the instance. If present then `instanceRequirements` cannot be present.
     */
    instanceType?: pulumi.Input<string>;
    /**
     * The kernel ID.
     */
    kernelId?: pulumi.Input<string>;
    /**
     * The key name to use for the instance.
     */
    keyName?: pulumi.Input<string>;
    /**
     * The latest version of the launch template.
     */
    latestVersion?: pulumi.Input<number>;
    /**
     * A list of license specifications to associate with. See License Specification below for more details.
     */
    licenseSpecifications?: pulumi.Input<pulumi.Input<inputs.ec2.LaunchTemplateLicenseSpecification>[]>;
    /**
     * The maintenance options for the instance. See Maintenance Options below for more details.
     */
    maintenanceOptions?: pulumi.Input<inputs.ec2.LaunchTemplateMaintenanceOptions>;
    /**
     * Customize the metadata options for the instance. See Metadata Options below for more details.
     */
    metadataOptions?: pulumi.Input<inputs.ec2.LaunchTemplateMetadataOptions>;
    /**
     * The monitoring option for the instance. See Monitoring below for more details.
     */
    monitoring?: pulumi.Input<inputs.ec2.LaunchTemplateMonitoring>;
    /**
     * The name of the launch template. If you leave this blank, the provider will auto-generate a unique name.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Customize network interfaces to be attached at instance boot time. See Network
     * Interfaces below for more details.
     */
    networkInterfaces?: pulumi.Input<pulumi.Input<inputs.ec2.LaunchTemplateNetworkInterface>[]>;
    /**
     * The placement of the instance. See Placement below for more details.
     */
    placement?: pulumi.Input<inputs.ec2.LaunchTemplatePlacement>;
    /**
     * The options for the instance hostname. The default values are inherited from the subnet. See Private DNS Name Options below for more details.
     */
    privateDnsNameOptions?: pulumi.Input<inputs.ec2.LaunchTemplatePrivateDnsNameOptions>;
    /**
     * The ID of the RAM disk.
     */
    ramDiskId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list of security group names to associate with. If you are creating Instances in a VPC, use
     * `vpcSecurityGroupIds` instead.
     */
    securityGroupNames?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The tags to apply to the resources during launch. See Tag Specifications below for more details. Default tags are currently not propagated to ASG created resources so you may wish to inject your default tags into this variable against the relevant child resource types created.
     */
    tagSpecifications?: pulumi.Input<pulumi.Input<inputs.ec2.LaunchTemplateTagSpecification>[]>;
    /**
     * A map of tags to assign to the launch template. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * Whether to update Default Version each update. Conflicts with `defaultVersion`.
     */
    updateDefaultVersion?: pulumi.Input<boolean>;
    /**
     * The base64-encoded user data to provide when launching the instance.
     */
    userData?: pulumi.Input<string>;
    /**
     * A list of security group IDs to associate with. Conflicts with `network_interfaces.security_groups`
     */
    vpcSecurityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
}
/**
 * The set of arguments for constructing a LaunchTemplate resource.
 */
export interface LaunchTemplateArgs {
    /**
     * Specify volumes to attach to the instance besides the volumes specified by the AMI.
     * See Block Devices below for details.
     */
    blockDeviceMappings?: pulumi.Input<pulumi.Input<inputs.ec2.LaunchTemplateBlockDeviceMapping>[]>;
    /**
     * Targeting for EC2 capacity reservations. See Capacity Reservation Specification below for more details.
     */
    capacityReservationSpecification?: pulumi.Input<inputs.ec2.LaunchTemplateCapacityReservationSpecification>;
    /**
     * The CPU options for the instance. See CPU Options below for more details.
     */
    cpuOptions?: pulumi.Input<inputs.ec2.LaunchTemplateCpuOptions>;
    /**
     * Customize the credit specification of the instance. See Credit
     * Specification below for more details.
     */
    creditSpecification?: pulumi.Input<inputs.ec2.LaunchTemplateCreditSpecification>;
    /**
     * Default Version of the launch template.
     */
    defaultVersion?: pulumi.Input<number>;
    /**
     * Description of the launch template.
     */
    description?: pulumi.Input<string>;
    /**
     * If true, enables [EC2 Instance Stop Protection](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-stop-protection.html).
     */
    disableApiStop?: pulumi.Input<boolean>;
    /**
     * If `true`, enables [EC2 Instance
     * Termination Protection](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_ChangingDisableAPITermination.html)
     */
    disableApiTermination?: pulumi.Input<boolean>;
    /**
     * If `true`, the launched EC2 instance will be EBS-optimized.
     */
    ebsOptimized?: pulumi.Input<string>;
    /**
     * Enable Nitro Enclaves on launched instances. See Enclave Options below for more details.
     */
    enclaveOptions?: pulumi.Input<inputs.ec2.LaunchTemplateEnclaveOptions>;
    /**
     * The hibernation options for the instance. See Hibernation Options below for more details.
     */
    hibernationOptions?: pulumi.Input<inputs.ec2.LaunchTemplateHibernationOptions>;
    /**
     * The IAM Instance Profile to launch the instance with. See Instance Profile
     * below for more details.
     */
    iamInstanceProfile?: pulumi.Input<inputs.ec2.LaunchTemplateIamInstanceProfile>;
    /**
     * The AMI from which to launch the instance or use a Systems Manager parameter convention e.g. `resolve:ssm:parameter-name`. See [docs](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/create-launch-template.html#use-an-ssm-parameter-instead-of-an-ami-id) for more details.
     */
    imageId?: pulumi.Input<string>;
    /**
     * Shutdown behavior for the instance. Can be `stop` or `terminate`.
     * (Default: `stop`).
     */
    instanceInitiatedShutdownBehavior?: pulumi.Input<string>;
    /**
     * The market (purchasing) option for the instance. See Market Options
     * below for details.
     */
    instanceMarketOptions?: pulumi.Input<inputs.ec2.LaunchTemplateInstanceMarketOptions>;
    /**
     * The attribute requirements for the type of instance. If present then `instanceType` cannot be present.
     */
    instanceRequirements?: pulumi.Input<inputs.ec2.LaunchTemplateInstanceRequirements>;
    /**
     * The type of the instance. If present then `instanceRequirements` cannot be present.
     */
    instanceType?: pulumi.Input<string>;
    /**
     * The kernel ID.
     */
    kernelId?: pulumi.Input<string>;
    /**
     * The key name to use for the instance.
     */
    keyName?: pulumi.Input<string>;
    /**
     * A list of license specifications to associate with. See License Specification below for more details.
     */
    licenseSpecifications?: pulumi.Input<pulumi.Input<inputs.ec2.LaunchTemplateLicenseSpecification>[]>;
    /**
     * The maintenance options for the instance. See Maintenance Options below for more details.
     */
    maintenanceOptions?: pulumi.Input<inputs.ec2.LaunchTemplateMaintenanceOptions>;
    /**
     * Customize the metadata options for the instance. See Metadata Options below for more details.
     */
    metadataOptions?: pulumi.Input<inputs.ec2.LaunchTemplateMetadataOptions>;
    /**
     * The monitoring option for the instance. See Monitoring below for more details.
     */
    monitoring?: pulumi.Input<inputs.ec2.LaunchTemplateMonitoring>;
    /**
     * The name of the launch template. If you leave this blank, the provider will auto-generate a unique name.
     */
    name?: pulumi.Input<string>;
    /**
     * Creates a unique name beginning with the specified prefix. Conflicts with `name`.
     */
    namePrefix?: pulumi.Input<string>;
    /**
     * Customize network interfaces to be attached at instance boot time. See Network
     * Interfaces below for more details.
     */
    networkInterfaces?: pulumi.Input<pulumi.Input<inputs.ec2.LaunchTemplateNetworkInterface>[]>;
    /**
     * The placement of the instance. See Placement below for more details.
     */
    placement?: pulumi.Input<inputs.ec2.LaunchTemplatePlacement>;
    /**
     * The options for the instance hostname. The default values are inherited from the subnet. See Private DNS Name Options below for more details.
     */
    privateDnsNameOptions?: pulumi.Input<inputs.ec2.LaunchTemplatePrivateDnsNameOptions>;
    /**
     * The ID of the RAM disk.
     */
    ramDiskId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A list of security group names to associate with. If you are creating Instances in a VPC, use
     * `vpcSecurityGroupIds` instead.
     */
    securityGroupNames?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The tags to apply to the resources during launch. See Tag Specifications below for more details. Default tags are currently not propagated to ASG created resources so you may wish to inject your default tags into this variable against the relevant child resource types created.
     */
    tagSpecifications?: pulumi.Input<pulumi.Input<inputs.ec2.LaunchTemplateTagSpecification>[]>;
    /**
     * A map of tags to assign to the launch template. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Whether to update Default Version each update. Conflicts with `defaultVersion`.
     */
    updateDefaultVersion?: pulumi.Input<boolean>;
    /**
     * The base64-encoded user data to provide when launching the instance.
     */
    userData?: pulumi.Input<string>;
    /**
     * A list of security group IDs to associate with. Conflicts with `network_interfaces.security_groups`
     */
    vpcSecurityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
}
