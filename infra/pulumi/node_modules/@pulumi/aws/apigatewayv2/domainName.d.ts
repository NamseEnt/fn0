import * as pulumi from "@pulumi/pulumi";
import * as inputs from "../types/input";
import * as outputs from "../types/output";
/**
 * Manages an Amazon API Gateway Version 2 domain name.
 * More information can be found in the [Amazon API Gateway Developer Guide](https://docs.aws.amazon.com/apigateway/latest/developerguide/how-to-custom-domains.html).
 *
 * > **Note:** This resource establishes ownership of and the TLS settings for
 * a particular domain name. An API stage can be associated with the domain name using the `aws.apigatewayv2.ApiMapping` resource.
 *
 * ## Example Usage
 *
 * ### Basic
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.apigatewayv2.DomainName("example", {
 *     domainName: "ws-api.example.com",
 *     domainNameConfiguration: {
 *         certificateArn: exampleAwsAcmCertificate.arn,
 *         endpointType: "REGIONAL",
 *         securityPolicy: "TLS_1_2",
 *     },
 * });
 * ```
 *
 * ### Associated Route 53 Resource Record
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = new aws.apigatewayv2.DomainName("example", {
 *     domainName: "http-api.example.com",
 *     domainNameConfiguration: {
 *         certificateArn: exampleAwsAcmCertificate.arn,
 *         endpointType: "REGIONAL",
 *         securityPolicy: "TLS_1_2",
 *     },
 * });
 * const exampleRecord = new aws.route53.Record("example", {
 *     name: example.domainName,
 *     type: aws.route53.RecordType.A,
 *     zoneId: exampleAwsRoute53Zone.zoneId,
 *     aliases: [{
 *         name: example.domainNameConfiguration.apply(domainNameConfiguration => domainNameConfiguration.targetDomainName),
 *         zoneId: example.domainNameConfiguration.apply(domainNameConfiguration => domainNameConfiguration.hostedZoneId),
 *         evaluateTargetHealth: false,
 *     }],
 * });
 * ```
 *
 * ## Import
 *
 * Using `pulumi import`, import `aws_apigatewayv2_domain_name` using the domain name. For example:
 *
 * ```sh
 * $ pulumi import aws:apigatewayv2/domainName:DomainName example ws-api.example.com
 * ```
 */
export declare class DomainName extends pulumi.CustomResource {
    /**
     * Get an existing DomainName resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: DomainNameState, opts?: pulumi.CustomResourceOptions): DomainName;
    /**
     * Returns true if the given object is an instance of DomainName.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is DomainName;
    /**
     * [API mapping selection expression](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api-selection-expressions.html#apigateway-websocket-api-mapping-selection-expressions) for the domain name.
     */
    readonly apiMappingSelectionExpression: pulumi.Output<string>;
    /**
     * ARN of the domain name.
     */
    readonly arn: pulumi.Output<string>;
    /**
     * Domain name. Must be between 1 and 512 characters in length.
     */
    readonly domainName: pulumi.Output<string>;
    /**
     * Domain name configuration. See below.
     */
    readonly domainNameConfiguration: pulumi.Output<outputs.apigatewayv2.DomainNameDomainNameConfiguration>;
    /**
     * Mutual TLS authentication configuration for the domain name.
     */
    readonly mutualTlsAuthentication: pulumi.Output<outputs.apigatewayv2.DomainNameMutualTlsAuthentication | undefined>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    readonly region: pulumi.Output<string>;
    /**
     * Map of tags to assign to the domain name. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    readonly tagsAll: pulumi.Output<{
        [key: string]: string;
    }>;
    /**
     * Create a DomainName resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args: DomainNameArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering DomainName resources.
 */
export interface DomainNameState {
    /**
     * [API mapping selection expression](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api-selection-expressions.html#apigateway-websocket-api-mapping-selection-expressions) for the domain name.
     */
    apiMappingSelectionExpression?: pulumi.Input<string>;
    /**
     * ARN of the domain name.
     */
    arn?: pulumi.Input<string>;
    /**
     * Domain name. Must be between 1 and 512 characters in length.
     */
    domainName?: pulumi.Input<string>;
    /**
     * Domain name configuration. See below.
     */
    domainNameConfiguration?: pulumi.Input<inputs.apigatewayv2.DomainNameDomainNameConfiguration>;
    /**
     * Mutual TLS authentication configuration for the domain name.
     */
    mutualTlsAuthentication?: pulumi.Input<inputs.apigatewayv2.DomainNameMutualTlsAuthentication>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the domain name. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * Map of tags assigned to the resource, including those inherited from the provider `defaultTags` configuration block.
     */
    tagsAll?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
/**
 * The set of arguments for constructing a DomainName resource.
 */
export interface DomainNameArgs {
    /**
     * Domain name. Must be between 1 and 512 characters in length.
     */
    domainName: pulumi.Input<string>;
    /**
     * Domain name configuration. See below.
     */
    domainNameConfiguration: pulumi.Input<inputs.apigatewayv2.DomainNameDomainNameConfiguration>;
    /**
     * Mutual TLS authentication configuration for the domain name.
     */
    mutualTlsAuthentication?: pulumi.Input<inputs.apigatewayv2.DomainNameMutualTlsAuthentication>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags to assign to the domain name. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
