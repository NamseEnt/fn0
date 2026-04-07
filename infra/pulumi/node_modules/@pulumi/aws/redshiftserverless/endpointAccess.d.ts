import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Creates a new Amazon Redshift Serverless Endpoint Access.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshiftserverless.EndpointAccess("example", {
 *     endpointName: "example",
 *     workgroupName: "example",
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift Serverless Endpoint Access using the `endpoint_name`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshiftserverless/endpointAccess:EndpointAccess example example
 * ```
 */
export declare class EndpointAccess extends pulumi.CustomResource {
    /**
     * Get an existing EndpointAccess resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: EndpointAccessState, opts?: pulumi.CustomResourceOptions): EndpointAccess;
    /**
     * Returns true if the given object is an instance of EndpointAccess.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is EndpointAccess;
    /**
     * The DNS address of the VPC endpoint.
     */
    readonly address: pulumi.Output<string>;
    /**
     * Amazon Resource Name (ARN) of the Redshift Serverless Endpoint Access.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * The name of the endpoint.
     */
    readonly endpointName: pulumi.Output<string>;
    /**
     * The owner Amazon Web Services account for the Amazon Redshift Serverless workgroup.
     */
    readonly ownerAccount: pulumi.Output<string | undefined>;
    /**
     * The port that Amazon Redshift Serverless listens on.
     */
    readonly port: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * An array of VPC subnet IDs to associate with the endpoint.
     */
    readonly subnetIds: pulumi.Output<string[]>;
    /**
     * The VPC endpoint or the Redshift Serverless workgroup. See `VPC Endpoint` below.
     */
    readonly vpcEndpoints: pulumi.Output<outputs.redshiftserverless.EndpointAccessVpcEndpoint[]>;
    /**
     * An array of security group IDs to associate with the workgroup.
     */
    readonly vpcSecurityGroupIds: pulumi.Output<string[]>;
    /**
     * The name of the workgroup.
     */
    readonly workgroupName: pulumi.Output<string>;
    /**
     * Create a EndpointAccess resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: EndpointAccessArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering EndpointAccess resources.
 */
export interface EndpointAccessState {
    /**
     * The DNS address of the VPC endpoint.
     */
    address?: pulumi.Input<string>;
    /**
     * Amazon Resource Name (ARN) of the Redshift Serverless Endpoint Access.
     */
    arn?: pulumi.Input<string>;
    /**
     * The name of the endpoint.
     */
    endpointName?: pulumi.Input<string>;
    /**
     * The owner Amazon Web Services account for the Amazon Redshift Serverless workgroup.
     */
    ownerAccount?: pulumi.Input<string>;
    /**
     * The port that Amazon Redshift Serverless listens on.
     */
    port?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * An array of VPC subnet IDs to associate with the endpoint.
     */
    subnetIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The VPC endpoint or the Redshift Serverless workgroup. See `VPC Endpoint` below.
     */
    vpcEndpoints?: pulumi.Input<pulumi.Input<inputs.redshiftserverless.EndpointAccessVpcEndpoint>[]>;
    /**
     * An array of security group IDs to associate with the workgroup.
     */
    vpcSecurityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name of the workgroup.
     */
    workgroupName?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a EndpointAccess resource.
 */
export interface EndpointAccessArgs {
    /**
     * The name of the endpoint.
     */
    endpointName: pulumi.Input<string>;
    /**
     * The owner Amazon Web Services account for the Amazon Redshift Serverless workgroup.
     */
    ownerAccount?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * An array of VPC subnet IDs to associate with the endpoint.
     */
    subnetIds: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * An array of security group IDs to associate with the workgroup.
     */
    vpcSecurityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * The name of the workgroup.
     */
    workgroupName: pulumi.Input<string>;
}
