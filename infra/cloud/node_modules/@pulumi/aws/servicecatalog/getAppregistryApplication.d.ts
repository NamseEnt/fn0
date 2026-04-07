import * as pulumi from "@pulumi/pulumi";
/**
 * Data source for managing an AWS Service Catalog AppRegistry Application.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.servicecatalog.getAppregistryApplication({
 *     id: "application-1234",
 * });
 * ```
 */
export declare function getAppregistryApplication(args: GetAppregistryApplicationArgs, opts?: pulumi.InvokeOptions): Promise<GetAppregistryApplicationResult>;
/**
 * A collection of arguments for invoking getAppregistryApplication.
 */
export interface GetAppregistryApplicationArgs {
    /**
     * Application identifier.
     */
    id: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getAppregistryApplication.
 */
export interface GetAppregistryApplicationResult {
    /**
     * A map with a single tag key-value pair used to associate resources with the application.
     */
    readonly applicationTag: {
        [key: string]: string;
    };
    /**
     * ARN (Amazon Resource Name) of the application.
     */
    readonly arn: string;
    /**
     * Description of the application.
     */
    readonly description: string;
    readonly id: string;
    /**
     * Name of the application.
     */
    readonly name: string;
    readonly region: string;
    /**
     * A map of tags assigned to the Application. If configured with a provider `defaultTags` configuration block present, tags with matching keys will overwrite those defined at the provider-level.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Data source for managing an AWS Service Catalog AppRegistry Application.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.servicecatalog.getAppregistryApplication({
 *     id: "application-1234",
 * });
 * ```
 */
export declare function getAppregistryApplicationOutput(args: GetAppregistryApplicationOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetAppregistryApplicationResult>;
/**
 * A collection of arguments for invoking getAppregistryApplication.
 */
export interface GetAppregistryApplicationOutputArgs {
    /**
     * Application identifier.
     */
    id: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
