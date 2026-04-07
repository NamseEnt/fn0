import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Creates a new Amazon Redshift endpoint access.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.redshift.EndpointAccess("example", {
 *     endpointName: "example",
 *     subnetGroupName: exampleAwsRedshiftSubnetGroup.id,
 *     clusterIdentifier: exampleAwsRedshiftCluster.clusterIdentifier,
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import Redshift endpoint access using the `name`. For example:
 *
 * ```sh
 * $ pulumi import aws:redshift/endpointAccess:EndpointAccess example example
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
     * The DNS address of the endpoint.
     */
    readonly address: pulumi.Output<string>;
    /**
     * The cluster identifier of the cluster to access.
     */
    readonly clusterIdentifier: pulumi.Output<string>;
    /**
     * The Redshift-managed VPC endpoint name.
     */
    readonly endpointName: pulumi.Output<string>;
    /**
     * The port number on which the cluster accepts incoming connections.
     */
    readonly port: pulumi.Output<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Web Services account ID of the owner of the cluster. This is only required if the cluster is in another Amazon Web Services account.
     */
    readonly resourceOwner: pulumi.Output<string>;
    /**
     * The subnet group from which Amazon Redshift chooses the subnet to deploy the endpoint.
     */
    readonly subnetGroupName: pulumi.Output<string>;
    /**
     * The connection endpoint for connecting to an Amazon Redshift cluster through the proxy. See details below.
     */
    readonly vpcEndpoints: pulumi.Output<outputs.redshift.EndpointAccessVpcEndpoint[]>;
    /**
     * The security group that defines the ports, protocols, and sources for inbound traffic that you are authorizing into your endpoint.
     */
    readonly vpcSecurityGroupIds: pulumi.Output<string[]>;
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
     * The DNS address of the endpoint.
     */
    address?: pulumi.Input<string>;
    /**
     * The cluster identifier of the cluster to access.
     */
    clusterIdentifier?: pulumi.Input<string>;
    /**
     * The Redshift-managed VPC endpoint name.
     */
    endpointName?: pulumi.Input<string>;
    /**
     * The port number on which the cluster accepts incoming connections.
     */
    port?: pulumi.Input<number>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Web Services account ID of the owner of the cluster. This is only required if the cluster is in another Amazon Web Services account.
     */
    resourceOwner?: pulumi.Input<string>;
    /**
     * The subnet group from which Amazon Redshift chooses the subnet to deploy the endpoint.
     */
    subnetGroupName?: pulumi.Input<string>;
    /**
     * The connection endpoint for connecting to an Amazon Redshift cluster through the proxy. See details below.
     */
    vpcEndpoints?: pulumi.Input<pulumi.Input<inputs.redshift.EndpointAccessVpcEndpoint>[]>;
    /**
     * The security group that defines the ports, protocols, and sources for inbound traffic that you are authorizing into your endpoint.
     */
    vpcSecurityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
}
/**
 * The set of arguments for constructing a EndpointAccess resource.
 */
export interface EndpointAccessArgs {
    /**
     * The cluster identifier of the cluster to access.
     */
    clusterIdentifier: pulumi.Input<string>;
    /**
     * The Redshift-managed VPC endpoint name.
     */
    endpointName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Web Services account ID of the owner of the cluster. This is only required if the cluster is in another Amazon Web Services account.
     */
    resourceOwner?: pulumi.Input<string>;
    /**
     * The subnet group from which Amazon Redshift chooses the subnet to deploy the endpoint.
     */
    subnetGroupName: pulumi.Input<string>;
    /**
     * The security group that defines the ports, protocols, and sources for inbound traffic that you are authorizing into your endpoint.
     */
    vpcSecurityGroupIds?: pulumi.Input<pulumi.Input<string>[]>;
}
