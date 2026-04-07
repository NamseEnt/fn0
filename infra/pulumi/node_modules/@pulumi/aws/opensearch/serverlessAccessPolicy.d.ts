import * as pulumi from "@pulumi/pulumi";
/**
 * Resource for managing an AWS OpenSearch Serverless Access Policy. See AWS documentation for [data access policies](https://docs.aws.amazon.com/opensearch-service/latest/developerguide/serverless-data-access.html) and [supported data access policy permissions](https://docs.aws.amazon.com/opensearch-service/latest/developerguide/serverless-data-access.html#serverless-data-supported-permissions).
 *
 * ## Example Usage
 *
 * ### Grant all collection and index permissions
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const example = new aws.opensearch.ServerlessAccessPolicy("example", {
 *     name: "example",
 *     type: "data",
 *     description: "read and write permissions",
 *     policy: JSON.stringify([{
 *         Rules: [
 *             {
 *                 ResourceType: "index",
 *                 Resource: ["index/example-collection/*"],
 *                 Permission: ["aoss:*"],
 *             },
 *             {
 *                 ResourceType: "collection",
 *                 Resource: ["collection/example-collection"],
 *                 Permission: ["aoss:*"],
 *             },
 *         ],
 *         Principal: [current.then(current => current.arn)],
 *     }]),
 * });
 * ```
 *
 * ### Grant read-only collection and index permissions
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const current = aws.getCallerIdentity({});
 * const example = new aws.opensearch.ServerlessAccessPolicy("example", {
 *     name: "example",
 *     type: "data",
 *     description: "read-only permissions",
 *     policy: JSON.stringify([{
 *         Rules: [
 *             {
 *                 ResourceType: "index",
 *                 Resource: ["index/example-collection/*"],
 *                 Permission: [
 *                     "aoss:DescribeIndex",
 *                     "aoss:ReadDocument",
 *                 ],
 *             },
 *             {
 *                 ResourceType: "collection",
 *                 Resource: ["collection/example-collection"],
 *                 Permission: ["aoss:DescribeCollectionItems"],
 *             },
 *         ],
 *         Principal: [current.then(current => current.arn)],
 *     }]),
 * });
 * ```
 *
 * ### Grant SAML identity permissions
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.opensearch.ServerlessAccessPolicy("example", {
 *     name: "example",
 *     type: "data",
 *     description: "saml permissions",
 *     policy: JSON.stringify([{
 *         Rules: [
 *             {
 *                 ResourceType: "index",
 *                 Resource: ["index/example-collection/*"],
 *                 Permission: ["aoss:*"],
 *             },
 *             {
 *                 ResourceType: "collection",
 *                 Resource: ["collection/example-collection"],
 *                 Permission: ["aoss:*"],
 *             },
 *         ],
 *         Principal: [
 *             "saml/123456789012/myprovider/user/Annie",
 *             "saml/123456789012/anotherprovider/group/Accounting",
 *         ],
 *     }]),
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import OpenSearchServerless Access Policy using the `name` and `type` arguments separated by a slash (`/`). For example:
 *
 * ```sh
 * $ pulumi import aws:opensearch/serverlessAccessPolicy:ServerlessAccessPolicy example example/data
 * ```
 */
export declare class ServerlessAccessPolicy extends pulumi.CustomResource {
    /**
     * Get an existing ServerlessAccessPolicy resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: ServerlessAccessPolicyState, opts?: pulumi.CustomResourceOptions): ServerlessAccessPolicy;
    /**
     * Returns true if the given object is an instance of ServerlessAccessPolicy.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is ServerlessAccessPolicy;
    /**
     * Description of the policy. Typically used to store information about the permissions defined in the policy.
     */
    readonly description: pulumi.Output<string | undefined>;
    /**
     * Name of the policy.
     */
    readonly name: pulumi.Output<string>;
    /**
     * JSON policy document to use as the content for the new policy
     */
    readonly policy: pulumi.Output<string>;
    /**
     * Version of the policy.
     */
    readonly policyVersion: pulumi.Output<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Type of access policy. Must be `data`.
     *
     * The following arguments are optional:
     */
    readonly type: pulumi.Output<string>;
    /**
     * Create a ServerlessAccessPolicy resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: ServerlessAccessPolicyArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering ServerlessAccessPolicy resources.
 */
export interface ServerlessAccessPolicyState {
    /**
     * Description of the policy. Typically used to store information about the permissions defined in the policy.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the policy.
     */
    name?: pulumi.Input<string>;
    /**
     * JSON policy document to use as the content for the new policy
     */
    policy?: pulumi.Input<string>;
    /**
     * Version of the policy.
     */
    policyVersion?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Type of access policy. Must be `data`.
     *
     * The following arguments are optional:
     */
    type?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a ServerlessAccessPolicy resource.
 */
export interface ServerlessAccessPolicyArgs {
    /**
     * Description of the policy. Typically used to store information about the permissions defined in the policy.
     */
    description?: pulumi.Input<string>;
    /**
     * Name of the policy.
     */
    name?: pulumi.Input<string>;
    /**
     * JSON policy document to use as the content for the new policy
     */
    policy: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Type of access policy. Must be `data`.
     *
     * The following arguments are optional:
     */
    type: pulumi.Input<string>;
}
