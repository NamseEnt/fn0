import * as pulumi from "@pulumi/pulumi";
/**
 * Creates a WAFv2 Web ACL Association.
 *
 * > **NOTE on associating a WAFv2 Web ACL with a Cloudfront distribution:** Do not use this resource to associate a WAFv2 Web ACL with a Cloudfront Distribution. The [AWS API call backing this resource](https://docs.aws.amazon.com/waf/latest/APIReference/API_AssociateWebACL.html) notes that you should use the `webAclId` property on the `cloudfrontDistribution` instead.
 *
 * [1]: https://docs.aws.amazon.com/waf/latest/APIReference/API_AssociateWebACL.html
 *
 * ## Import
 *
 * Using `pulumi import`, import WAFv2 Web ACL Association using `WEB_ACL_ARN,RESOURCE_ARN`. For example:
 *
 * ```sh
 * $ pulumi import aws:wafv2/webAclAssociation:WebAclAssociation example arn:aws:wafv2:...7ce849ea,arn:aws:apigateway:...ages/name
 * ```
 */
export declare class WebAclAssociation extends pulumi.CustomResource {
    /**
     * Get an existing WebAclAssociation resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: WebAclAssociationState, opts?: pulumi.CustomResourceOptions): WebAclAssociation;
    /**
     * Returns true if the given object is an instance of WebAclAssociation.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is WebAclAssociation;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the resource to associate with the web ACL. This must be an ARN of an Application Load Balancer, an Amazon API Gateway stage (REST only, HTTP is unsupported), an Amazon Cognito User Pool, an Amazon AppSync GraphQL API, an Amazon App Runner service, or an Amazon Verified Access instance.
     */
    readonly resourceArn: pulumi.Output<string>;
    /**
     * The Amazon Resource Name (ARN) of the Web ACL that you want to associate with the resource.
     */
    readonly webAclArn: pulumi.Output<string>;
    /**
     * Create a WebAclAssociation resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: WebAclAssociationArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering WebAclAssociation resources.
 */
export interface WebAclAssociationState {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the resource to associate with the web ACL. This must be an ARN of an Application Load Balancer, an Amazon API Gateway stage (REST only, HTTP is unsupported), an Amazon Cognito User Pool, an Amazon AppSync GraphQL API, an Amazon App Runner service, or an Amazon Verified Access instance.
     */
    resourceArn?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the Web ACL that you want to associate with the resource.
     */
    webAclArn?: pulumi.Input<string>;
}
/**
 * The set of arguments for constructing a WebAclAssociation resource.
 */
export interface WebAclAssociationArgs {
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the resource to associate with the web ACL. This must be an ARN of an Application Load Balancer, an Amazon API Gateway stage (REST only, HTTP is unsupported), an Amazon Cognito User Pool, an Amazon AppSync GraphQL API, an Amazon App Runner service, or an Amazon Verified Access instance.
     */
    resourceArn: pulumi.Input<string>;
    /**
     * The Amazon Resource Name (ARN) of the Web ACL that you want to associate with the resource.
     */
    webAclArn: pulumi.Input<string>;
}
