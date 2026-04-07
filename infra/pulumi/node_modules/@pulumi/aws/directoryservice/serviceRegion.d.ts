import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a replicated Region and directory for Multi-Region replication.
 * Multi-Region replication is only supported for the Enterprise Edition of AWS Managed Microsoft AD.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 * import * as std from "@pulumi/std";
 *
 * const example = aws.getRegion({});
 * const available = aws.getAvailabilityZones({
 *     state: "available",
 *     filters: [{
 *         name: "opt-in-status",
 *         values: ["opt-in-not-required"],
 *     }],
 * });
 * const exampleVpc = new aws.ec2.Vpc("example", {
 *     cidrBlock: "10.0.0.0/16",
 *     tags: {
 *         Name: "Primary",
 *     },
 * });
 * const exampleSubnet: aws.ec2.Subnet[] = [];
 * for (const range = {value: 0}; range.value < 2; range.value++) {
 *     exampleSubnet.push(new aws.ec2.Subnet(`example-${range.value}`, {
 *         vpcId: exampleVpc.id,
 *         availabilityZone: available.then(available => available.names[range.value]),
 *         cidrBlock: exampleVpc.cidrBlock.apply(cidrBlock => std.cidrsubnetOutput({
 *             input: cidrBlock,
 *             newbits: 8,
 *             netnum: range.value,
 *         })).apply(invoke => invoke.result),
 *         tags: {
 *             Name: "Primary",
 *         },
 *     }));
 * }
 * const exampleDirectory = new aws.directoryservice.Directory("example", {
 *     name: "example.com",
 *     password: "SuperSecretPassw0rd",
 *     type: "MicrosoftAD",
 *     vpcSettings: {
 *         vpcId: exampleVpc.id,
 *         subnetIds: exampleSubnet.map(__item => __item.id),
 *     },
 * });
 * const available_secondary = aws.getAvailabilityZones({
 *     state: "available",
 *     filters: [{
 *         name: "opt-in-status",
 *         values: ["opt-in-not-required"],
 *     }],
 * });
 * const example_secondary = new aws.ec2.Vpc("example-secondary", {
 *     cidrBlock: "10.1.0.0/16",
 *     tags: {
 *         Name: "Secondary",
 *     },
 * });
 * const example_secondarySubnet: aws.ec2.Subnet[] = [];
 * for (const range = {value: 0}; range.value < 2; range.value++) {
 *     example_secondarySubnet.push(new aws.ec2.Subnet(`example-secondary-${range.value}`, {
 *         vpcId: example_secondary.id,
 *         availabilityZone: available_secondary.then(available_secondary => available_secondary.names[range.value]),
 *         cidrBlock: example_secondary.cidrBlock.apply(cidrBlock => std.cidrsubnetOutput({
 *             input: cidrBlock,
 *             newbits: 8,
 *             netnum: range.value,
 *         })).apply(invoke => invoke.result),
 *         tags: {
 *             Name: "Secondary",
 *         },
 *     }));
 * }
 * const exampleServiceRegion = new aws.directoryservice.ServiceRegion("example", {
 *     directoryId: exampleDirectory.id,
 *     regionName: example.then(example => example.name),
 *     vpcSettings: {
 *         vpcId: example_secondary.id,
 *         subnetIds: example_secondarySubnet.map(__item => __item.id),
 *     },
 *     tags: {
 *         Name: "Secondary",
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Replicated Regions using directory ID,Region name. For example:
 *
 * ```sh
 * $ pulumi import aws:directoryservice/serviceRegion:ServiceRegion example d-9267651497,us-east-2
 * ```
 */
export declare class ServiceRegion extends pulumi.CustomResource {
    /**
     * Get an existing ServiceRegion resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ServiceRegionState, opts?: pulumi.CustomResourceOptions): ServiceRegion;
    /**
     * Returns true if the given object is an instance of ServiceRegion.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ServiceRegion;
    /**
     * The number of domain controllers desired in the replicated directory. Minimum value of `2`.
     */
    readonly desiredNumberOfDomainControllers: pulumi.Output<number>;
    /**
     * The identifier of the directory to which you want to add Region replication.
     */
    readonly directoryId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The name of the Region where you want to add domain controllers for replication.
     */
    readonly regionName: pulumi.Output<string>;
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
     * VPC information in the replicated Region. Detailed below.
     */
    readonly vpcSettings: pulumi.Output<outputs.directoryservice.ServiceRegionVpcSettings>;
    /**
     * Create a ServiceRegion resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ServiceRegionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ServiceRegion resources.
 */
export interface ServiceRegionState {
    /**
     * The number of domain controllers desired in the replicated directory. Minimum value of `2`.
     */
    desiredNumberOfDomainControllers?: pulumi.Input<number>;
    /**
     * The identifier of the directory to which you want to add Region replication.
     */
    directoryId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the Region where you want to add domain controllers for replication.
     */
    regionName?: pulumi.Input<string>;
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
     * VPC information in the replicated Region. Detailed below.
     */
    vpcSettings?: pulumi.Input<inputs.directoryservice.ServiceRegionVpcSettings>;
}
/**
 * The set of arguments for constructing a ServiceRegion resource.
 */
export interface ServiceRegionArgs {
    /**
     * The number of domain controllers desired in the replicated directory. Minimum value of `2`.
     */
    desiredNumberOfDomainControllers?: pulumi.Input<number>;
    /**
     * The identifier of the directory to which you want to add Region replication.
     */
    directoryId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The name of the Region where you want to add domain controllers for replication.
     */
    regionName: pulumi.Input<string>;
    /**
     * Map of tags to assign to this resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * VPC information in the replicated Region. Detailed below.
     */
    vpcSettings: pulumi.Input<inputs.directoryservice.ServiceRegionVpcSettings>;
}
