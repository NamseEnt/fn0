import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS CodeConnections Host.
 *
 * > **NOTE:** The `aws.codeconnections.Host` resource is created in the state `PENDING`. Authentication with the host provider must be completed in the AWS Console. For more information visit [Set up a pending host](https://docs.aws.amazon.com/dtconsole/latest/userguide/connections-host-setup.html).
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.codeconnections.Host("example", {
 *     name: "example-host",
 *     providerEndpoint: "https://example.com",
 *     providerType: "GitHubEnterpriseServer",
 * });
 * ```
 *
 * ## Import
 *
 * ### Identity Schema
 *
 * #### Required
 *
 * - `arn` (String) Amazon Resource Name (ARN) of the CodeConnections host.
 *
 * Using `pulumi import`, import CodeConnections Host using the ARN. For example:
 *
 * console
 *
 * % pulumi import aws_codeconnections_host.example-host arn:aws:codeconnections:us-west-1:0123456789:host/79d4d357-a2ee-41e4-b350-2fe39ae59448
 */
export declare class Host extends pulumi.CustomResource {
    /**
     * Get an existing Host resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: HostState, opts?: pulumi.CustomResourceOptions): Host;
    /**
     * Returns true if the given object is an instance of Host.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is Host;
    /**
     * The CodeConnections Host ARN.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The name of the host to be created. The name must be unique in the calling AWS account.
     */
    readonly name: pulumi.Output<string>;
    /**
     * The endpoint of the infrastructure to be represented by the host after it is created.
     */
    readonly providerEndpoint: pulumi.Output<string>;
    /**
     * The name of the external provider where your third-party code repository is configured.
     */
    readonly providerType: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    readonly timeouts: pulumi.Output<outputs.codeconnections.HostTimeouts | undefined>;
    /**
     * The VPC configuration to be provisioned for the host. A VPC must be configured, and the infrastructure to be represented by the host must already be connected to the VPC.
     */
    readonly vpcConfiguration: pulumi.Output<outputs.codeconnections.HostVpcConfiguration | undefined>;
    /**
     * Create a Host resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: HostArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering Host resources.
 */
export interface HostState {
    /**
     * The CodeConnections Host ARN.
     */
    arn?: pulumi.Input<string>;
    /**
     * The name of the host to be created. The name must be unique in the calling AWS account.
     */
    name?: pulumi.Input<string>;
    /**
     * The endpoint of the infrastructure to be represented by the host after it is created.
     */
    providerEndpoint?: pulumi.Input<string>;
    /**
     * The name of the external provider where your third-party code repository is configured.
     */
    providerType?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.codeconnections.HostTimeouts>;
    /**
     * The VPC configuration to be provisioned for the host. A VPC must be configured, and the infrastructure to be represented by the host must already be connected to the VPC.
     */
    vpcConfiguration?: pulumi.Input<inputs.codeconnections.HostVpcConfiguration>;
}
/**
 * The set of arguments for constructing a Host resource.
 */
export interface HostArgs {
    /**
     * The name of the host to be created. The name must be unique in the calling AWS account.
     */
    name?: pulumi.Input<string>;
    /**
     * The endpoint of the infrastructure to be represented by the host after it is created.
     */
    providerEndpoint: pulumi.Input<string>;
    /**
     * The name of the external provider where your third-party code repository is configured.
     */
    providerType: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    timeouts?: pulumi.Input<inputs.codeconnections.HostTimeouts>;
    /**
     * The VPC configuration to be provisioned for the host. A VPC must be configured, and the infrastructure to be represented by the host must already be connected to the VPC.
     */
    vpcConfiguration?: pulumi.Input<inputs.codeconnections.HostVpcConfiguration>;
}
