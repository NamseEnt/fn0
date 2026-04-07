import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages a Lightsail container service deployment version. Use this resource to deploy containerized applications to your Lightsail container service with specific container configurations and settings.
 *
 * > **NOTE:** The Amazon Lightsail container service must be enabled to create a deployment.
 *
 * > **NOTE:** This resource allows you to manage an Amazon Lightsail container service deployment version but the provider cannot destroy it. Removing this resource from your configuration will remove it from your statefile.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.lightsail.ContainerServiceDeploymentVersion("example", {
 *     containers: [{
 *         containerName: "hello-world",
 *         image: "amazon/amazon-lightsail:hello-world",
 *         commands: [],
 *         environment: {
 *             MY_ENVIRONMENT_VARIABLE: "my_value",
 *         },
 *         ports: {
 *             "80": "HTTP",
 *         },
 *     }],
 *     publicEndpoint: {
 *         containerName: "hello-world",
 *         containerPort: 80,
 *         healthCheck: {
 *             healthyThreshold: 2,
 *             unhealthyThreshold: 2,
 *             timeoutSeconds: 2,
 *             intervalSeconds: 5,
 *             path: "/",
 *             successCodes: "200-499",
 *         },
 *     },
 *     serviceName: exampleAwsLightsailContainerService.name,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Lightsail Container Service Deployment Version using the `service_name` and `version` separated by a slash (`/`). For example:
 *
 * ```sh
 * $ pulumi import aws:lightsail/containerServiceDeploymentVersion:ContainerServiceDeploymentVersion example container-service-1/1
 * ```
 */
export declare class ContainerServiceDeploymentVersion extends pulumi.CustomResource {
    /**
     * Get an existing ContainerServiceDeploymentVersion resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ContainerServiceDeploymentVersionState, opts?: pulumi.CustomResourceOptions): ContainerServiceDeploymentVersion;
    /**
     * Returns true if the given object is an instance of ContainerServiceDeploymentVersion.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ContainerServiceDeploymentVersion;
    /**
     * Set of configuration blocks that describe the settings of the containers that will be launched on the container service. Maximum of 53. See below.
     */
    readonly containers: pulumi.Output<outputs.lightsail.ContainerServiceDeploymentVersionContainer[]>;
    /**
     * Date and time when the deployment was created.
     */
    readonly createdAt: pulumi.Output<string>;
    /**
     * Configuration block that describes the settings of the public endpoint for the container service. See below.
     */
    readonly publicEndpoint: pulumi.Output<outputs.lightsail.ContainerServiceDeploymentVersionPublicEndpoint | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Name of the container service.
     *
     * The following arguments are optional:
     */
    readonly serviceName: pulumi.Output<string>;
    /**
     * Current state of the container service.
     */
    readonly state: pulumi.Output<string>;
    /**
     * Version number of the deployment.
     */
    readonly version: pulumi.Output<number>;
    /**
     * Create a ContainerServiceDeploymentVersion resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ContainerServiceDeploymentVersionArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ContainerServiceDeploymentVersion resources.
 */
export interface ContainerServiceDeploymentVersionState {
    /**
     * Set of configuration blocks that describe the settings of the containers that will be launched on the container service. Maximum of 53. See below.
     */
    containers?: pulumi.Input<pulumi.Input<inputs.lightsail.ContainerServiceDeploymentVersionContainer>[]>;
    /**
     * Date and time when the deployment was created.
     */
    createdAt?: pulumi.Input<string>;
    /**
     * Configuration block that describes the settings of the public endpoint for the container service. See below.
     */
    publicEndpoint?: pulumi.Input<inputs.lightsail.ContainerServiceDeploymentVersionPublicEndpoint>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the container service.
     *
     * The following arguments are optional:
     */
    serviceName?: pulumi.Input<string>;
    /**
     * Current state of the container service.
     */
    state?: pulumi.Input<string>;
    /**
     * Version number of the deployment.
     */
    version?: pulumi.Input<number>;
}
/**
 * The set of arguments for constructing a ContainerServiceDeploymentVersion resource.
 */
export interface ContainerServiceDeploymentVersionArgs {
    /**
     * Set of configuration blocks that describe the settings of the containers that will be launched on the container service. Maximum of 53. See below.
     */
    containers: pulumi.Input<pulumi.Input<inputs.lightsail.ContainerServiceDeploymentVersionContainer>[]>;
    /**
     * Configuration block that describes the settings of the public endpoint for the container service. See below.
     */
    publicEndpoint?: pulumi.Input<inputs.lightsail.ContainerServiceDeploymentVersionPublicEndpoint>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Name of the container service.
     *
     * The following arguments are optional:
     */
    serviceName: pulumi.Input<string>;
}
