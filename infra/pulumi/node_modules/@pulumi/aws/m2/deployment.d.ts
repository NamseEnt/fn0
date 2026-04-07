import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an [AWS Mainframe Modernization Deployment.](https://docs.aws.amazon.com/m2/latest/userguide/applications-m2-deploy.html)
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = new aws.m2.Deployment("test", {
 *     environmentId: "01234567890abcdef012345678",
 *     applicationId: "34567890abcdef012345678012",
 *     applicationVersion: 1,
 *     start: true,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Mainframe Modernization Deployment using the `APPLICATION-ID,DEPLOYMENT-ID`. For example:
 *
 * ```sh
 * $ pulumi import aws:m2/deployment:Deployment example APPLICATION-ID,DEPLOYMENT-ID
 * ```
 */
export declare class Deployment extends pulumi.CustomResource {
    /**
     * Get an existing Deployment resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DeploymentState, opts?: pulumi.CustomResourceOptions): Deployment;
    /**
     * Returns true if the given object is an instance of Deployment.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Deployment;
    /**
     * Application to deploy.
     */
    readonly applicationId: pulumi.Output<string>;
    /**
     * Version to application to deploy
     */
    readonly applicationVersion: pulumi.Output<number>;
    readonly deploymentId: pulumi.Output<string>;
    /**
     * Environment to deploy application to.
     */
    readonly environmentId: pulumi.Output<string>;
    readonly forceStop: pulumi.Output<boolean | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Start the application once deployed.
     */
    readonly start: pulumi.Output<boolean>;
    readonly timeouts: pulumi.Output<outputs.m2.DeploymentTimeouts | undefined>;
    /**
     * Create a Deployment resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DeploymentArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Deployment resources.
 */
export interface DeploymentState {
    /**
     * Application to deploy.
     */
    applicationId?: pulumi.Input<string>;
    /**
     * Version to application to deploy
     */
    applicationVersion?: pulumi.Input<number>;
    deploymentId?: pulumi.Input<string>;
    /**
     * Environment to deploy application to.
     */
    environmentId?: pulumi.Input<string>;
    forceStop?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Start the application once deployed.
     */
    start?: pulumi.Input<boolean>;
    timeouts?: pulumi.Input<inputs.m2.DeploymentTimeouts>;
}
/**
 * The set of arguments for constructing a Deployment resource.
 */
export interface DeploymentArgs {
    /**
     * Application to deploy.
     */
    applicationId: pulumi.Input<string>;
    /**
     * Version to application to deploy
     */
    applicationVersion: pulumi.Input<number>;
    /**
     * Environment to deploy application to.
     */
    environmentId: pulumi.Input<string>;
    forceStop?: pulumi.Input<boolean>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Start the application once deployed.
     */
    start: pulumi.Input<boolean>;
    timeouts?: pulumi.Input<inputs.m2.DeploymentTimeouts>;
}
