import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Provides a resource to manage AWS Device Farm Test Grid Projects.
 *
 * > **NOTE:** AWS currently has limited regional support for Device Farm (e.g., `us-west-2`). See [AWS Device Farm endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/devicefarm.html) for information on supported regions.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.devicefarm.TestGridProject("example", {
 *     name: "example",
 *     vpcConfig: {
 *         vpcId: exampleAwsVpc.id,
 *         subnetIds: exampleAwsSubnet.map(__item => __item.id),
 *         securityGroupIds: exampleAwsSecurityGroup.map(__item => __item.id),
 *     },
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the Device Farm test grid project.
 *
 * Using `pulumi import`, import DeviceFarm Test Grid Projects using their ARN. For example:
 *
 * console
 *
 * % pulumi import aws_devicefarm_test_grid_project.example arn:aws:devicefarm:us-west-2:123456789012:testgrid-project:4fa784c7-ccb4-4dbf-ba4f-02198320daa1
 */
export declare class TestGridProject extends pulumi.CustomResource {
    /**
     * Get an existing TestGridProject resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: TestGridProjectState, opts?: pulumi.CustomResourceOptions): TestGridProject;
    /**
     * Returns true if the given object is an instance of TestGridProject.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is TestGridProject;
    /**
     * The Amazon Resource Name of this Test Grid Project.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Human-readable description of the project.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * The name of the Selenium testing project.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The VPC security groups and subnets that are attached to a project. See VPC Config below.
     */
    readonly vpcConfig: pulumi.Output<outputs.devicefarm.TestGridProjectVpcConfig | undefined>;
    /**
     * Create a TestGridProject resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: TestGridProjectArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering TestGridProject resources.
 */
export interface TestGridProjectState {
    /**
     * The Amazon Resource Name of this Test Grid Project.
     */
    arn?: pulumi.Input<string>;
    /**
     * Human-readable description of the project.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the Selenium testing project.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
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
     * The VPC security groups and subnets that are attached to a project. See VPC Config below.
     */
    vpcConfig?: pulumi.Input<inputs.devicefarm.TestGridProjectVpcConfig>;
}
/**
 * The set of arguments for constructing a TestGridProject resource.
 */
export interface TestGridProjectArgs {
    /**
     * Human-readable description of the project.
     */
    description?: pulumi.Input<string>;
    /**
     * The name of the Selenium testing project.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * A map of tags to assign to the resource. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The VPC security groups and subnets that are attached to a project. See VPC Config below.
     */
    vpcConfig?: pulumi.Input<inputs.devicefarm.TestGridProjectVpcConfig>;
}
