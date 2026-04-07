import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Resource for managing an AWS OpenSearch Authorize Vpc Endpoint Access.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const test = new aws.opensearch.AuthorizeVpcEndpointAccess("test", {
 *     domainName: testAwsOpensearchDomain.domainName,
 *     account: current.then(current => current.accountId),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import OpenSearch Authorize Vpc Endpoint Access using the `domain_name` and `account` separated by a comma (,). For example:
 *
 * ```sh
 * $ pulumi import aws:opensearch/authorizeVpcEndpointAccess:AuthorizeVpcEndpointAccess example authorize_vpc_endpoint_access-id-12345678,123456789012
 * ```
 */
export declare class AuthorizeVpcEndpointAccess extends pulumi.CustomResource {
    /**
     * Get an existing AuthorizeVpcEndpointAccess resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: AuthorizeVpcEndpointAccessState, opts?: pulumi.CustomResourceOptions): AuthorizeVpcEndpointAccess;
    /**
     * Returns true if the given object is an instance of AuthorizeVpcEndpointAccess.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is AuthorizeVpcEndpointAccess;
    /**
     * AWS account ID to grant access to.
     */
    readonly account: pulumi.Output<string>;
    /**
     * Information about the Amazon Web Services account or service that was provided access to the domain. See authorized principal attribute for further details.
     */
    readonly authorizedPrincipals: pulumi.Output<outputs.opensearch.AuthorizeVpcEndpointAccessAuthorizedPrincipal[]>;
    /**
     * Name of OpenSearch Service domain to provide access to.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Create a AuthorizeVpcEndpointAccess resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: AuthorizeVpcEndpointAccessArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering AuthorizeVpcEndpointAccess resources.
 */
export interface AuthorizeVpcEndpointAccessState {
    /**
     * AWS account ID to grant access to.
     */
    account?: pulumi.Input<string>;
    /**
     * Information about the Amazon Web Services account or service that was provided access to the domain. See authorized principal attribute for further details.
     */
    authorizedPrincipals?: pulumi.Input<pulumi.Input<inputs.opensearch.AuthorizeVpcEndpointAccessAuthorizedPrincipal>[]>;
    /**
     * Name of OpenSearch Service domain to provide access to.
     */
    domainName?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a AuthorizeVpcEndpointAccess resource.
 */
export interface AuthorizeVpcEndpointAccessArgs {
    /**
     * AWS account ID to grant access to.
     */
    account: pulumi.Input<string>;
    /**
     * Name of OpenSearch Service domain to provide access to.
     */
    domainName: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
