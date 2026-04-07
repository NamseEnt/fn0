import * as pulumi from "@pulumi/pulumi";
/**
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.servicediscovery.getHttpNamespace({
 *     name: "development",
 * });
 * ```
 */
export declare function getHttpNamespace(args: GetHttpNamespaceArgs, opts?: pulumi.InvokeOptions): Promise<GetHttpNamespaceResult>;
/**
 * A collection of arguments for invoking getHttpNamespace.
 */
export interface GetHttpNamespaceArgs {
    /**
     * Name of the http namespace.
     */
    name: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Map of tags for the resource.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getHttpNamespace.
 */
export interface GetHttpNamespaceResult {
    /**
     * ARN that Amazon Route 53 assigns to the namespace when you create it.
     */
    readonly arn: string;
    /**
     * Description that you specify for the namespace when you create it.
     */
    readonly description: string;
    /**
     * Name of an HTTP namespace.
     */
    readonly httpName: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
    readonly region: string;
    /**
     * Map of tags for the resource.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.servicediscovery.getHttpNamespace({
 *     name: "development",
 * });
 * ```
 */
export declare function getHttpNamespaceOutput(args: GetHttpNamespaceOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetHttpNamespaceResult>;
/**
 * A collection of arguments for invoking getHttpNamespace.
 */
export interface GetHttpNamespaceOutputArgs {
    /**
     * Name of the http namespace.
     */
    name: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Map of tags for the resource.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
