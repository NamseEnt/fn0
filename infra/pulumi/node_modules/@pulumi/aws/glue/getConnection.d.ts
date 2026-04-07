import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * This data source can be used to fetch information about a specific Glue Connection.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.glue.getConnection({
 *     id: "123456789123:connection",
 * });
 * ```
 */
export declare function getConnection(args: GetConnectionArgs, opts?: pulumi.InvokeOptions): Promise<GetConnectionResult>;
/**
 * A collection of arguments for invoking getConnection.
 */
export interface GetConnectionArgs {
    /**
     * Concatenation of the catalog ID and connection name. For example, if your account ID is
     * `123456789123` and the connection name is `conn` then the ID is `123456789123:conn`.
     */
    id: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * Tags assigned to the resource
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getConnection.
 */
export interface GetConnectionResult {
    /**
     * ARN of the Glue Connection.
     */
    readonly arn: string;
    /**
     * A map of connection properties specific to the Athena compute environment.
     */
    readonly athenaProperties: {
        [key: string]: string;
    };
    /**
     * Catalog ID of the Glue Connection.
     */
    readonly catalogId: string;
    /**
     * A map of connection properties.
     */
    readonly connectionProperties: {
        [key: string]: string;
    };
    /**
     * Type of Glue Connection.
     */
    readonly connectionType: string;
    /**
     * Description of the connection.
     */
    readonly description: string;
    readonly id: string;
    /**
     * A list of criteria that can be used in selecting this connection.
     */
    readonly matchCriterias: string[];
    /**
     * Name of the Glue Connection.
     */
    readonly name: string;
    /**
     * A map of physical connection requirements, such as VPC and SecurityGroup.
     */
    readonly physicalConnectionRequirements: outputs.glue.GetConnectionPhysicalConnectionRequirement[];
    readonly region: string;
    /**
     * Tags assigned to the resource
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * This data source can be used to fetch information about a specific Glue Connection.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.glue.getConnection({
 *     id: "123456789123:connection",
 * });
 * ```
 */
export declare function getConnectionOutput(args: GetConnectionOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetConnectionResult>;
/**
 * A collection of arguments for invoking getConnection.
 */
export interface GetConnectionOutputArgs {
    /**
     * Concatenation of the catalog ID and connection name. For example, if your account ID is
     * `123456789123` and the connection name is `conn` then the ID is `123456789123:conn`.
     */
    id: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * Tags assigned to the resource
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
