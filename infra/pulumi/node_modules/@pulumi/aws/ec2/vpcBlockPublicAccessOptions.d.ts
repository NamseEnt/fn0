import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS VPC Block Public Access Options.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2.VpcBlockPublicAccessOptions("example", {internetGatewayBlockMode: "block-bidirectional"});
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import VPC Block Public Access Options using the `aws_region`. For example:
 *
 * ```sh
 * $ pulumi import aws:ec2/vpcBlockPublicAccessOptions:VpcBlockPublicAccessOptions example us-east-1
 * ```
 */
export declare class VpcBlockPublicAccessOptions extends pulumi.CustomResource {
    /**
     * Get an existing VpcBlockPublicAccessOptions resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: VpcBlockPublicAccessOptionsState, opts?: pulumi.CustomResourceOptions): VpcBlockPublicAccessOptions;
    /**
     * Returns true if the given object is an instance of VpcBlockPublicAccessOptions.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is VpcBlockPublicAccessOptions;
    /**
     * The AWS account id to which these options apply.
     */
    readonly awsAccountId: pulumi.Output<string>;
    /**
     * The AWS region to which these options apply.
     */
    readonly awsRegion: pulumi.Output<string>;
    /**
     * Block mode. Needs to be one of `block-bidirectional`, `block-ingress`, `off`. If this resource is deleted, then this value will be set to `off` in the AWS account and region.
     */
    readonly internetGatewayBlockMode: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.ec2.VpcBlockPublicAccessOptionsTimeouts | undefined>;
    /**
     * Create a VpcBlockPublicAccessOptions resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: VpcBlockPublicAccessOptionsArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering VpcBlockPublicAccessOptions resources.
 */
export interface VpcBlockPublicAccessOptionsState {
    /**
     * The AWS account id to which these options apply.
     */
    awsAccountId?: pulumi.Input<string>;
    /**
     * The AWS region to which these options apply.
     */
    awsRegion?: pulumi.Input<string>;
    /**
     * Block mode. Needs to be one of `block-bidirectional`, `block-ingress`, `off`. If this resource is deleted, then this value will be set to `off` in the AWS account and region.
     */
    internetGatewayBlockMode?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.ec2.VpcBlockPublicAccessOptionsTimeouts>;
}
/**
 * The set of arguments for constructing a VpcBlockPublicAccessOptions resource.
 */
export interface VpcBlockPublicAccessOptionsArgs {
    /**
     * Block mode. Needs to be one of `block-bidirectional`, `block-ingress`, `off`. If this resource is deleted, then this value will be set to `off` in the AWS account and region.
     */
    internetGatewayBlockMode: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.ec2.VpcBlockPublicAccessOptionsTimeouts>;
}
