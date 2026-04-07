import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS EC2 (Elastic Compute Cloud) Transit Gateway Default Route Table Propagation.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.ec2transitgateway.DefaultRouteTablePropagation("example", {
 *     transitGatewayId: exampleAwsEc2TransitGateway.id,
 *     transitGatewayRouteTableId: exampleAwsEc2TransitGatewayRouteTable.id,
 * });
 * ```
 */
export declare class DefaultRouteTablePropagation extends pulumi.CustomResource {
    /**
     * Get an existing DefaultRouteTablePropagation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DefaultRouteTablePropagationState, opts?: pulumi.CustomResourceOptions): DefaultRouteTablePropagation;
    /**
     * Returns true if the given object is an instance of DefaultRouteTablePropagation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DefaultRouteTablePropagation;
    readonly originalDefaultRouteTableId: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    readonly timeouts: pulumi.Output<outputs.ec2transitgateway.DefaultRouteTablePropagationTimeouts | undefined>;
    /**
     * ID of the Transit Gateway to change the default association route table on.
     */
    readonly transitGatewayId: pulumi.Output<string>;
    /**
     * ID of the Transit Gateway Route Table to be made the default association route table.
     */
    readonly transitGatewayRouteTableId: pulumi.Output<string>;
    /**
     * Create a DefaultRouteTablePropagation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DefaultRouteTablePropagationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DefaultRouteTablePropagation resources.
 */
export interface DefaultRouteTablePropagationState {
    originalDefaultRouteTableId?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.ec2transitgateway.DefaultRouteTablePropagationTimeouts>;
    /**
     * ID of the Transit Gateway to change the default association route table on.
     */
    transitGatewayId?: pulumi.Input<string>;
    /**
     * ID of the Transit Gateway Route Table to be made the default association route table.
     */
    transitGatewayRouteTableId?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a DefaultRouteTablePropagation resource.
 */
export interface DefaultRouteTablePropagationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    timeouts?: pulumi.Input<inputs.ec2transitgateway.DefaultRouteTablePropagationTimeouts>;
    /**
     * ID of the Transit Gateway to change the default association route table on.
     */
    transitGatewayId: pulumi.Input<string>;
    /**
     * ID of the Transit Gateway Route Table to be made the default association route table.
     */
    transitGatewayRouteTableId: pulumi.Input<string>;
}
