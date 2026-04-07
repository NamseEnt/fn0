import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS DMS (Database Migration) Endpoint.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.dms.getEndpoint({
 *     endpointId: "test_id",
 * });
 * ```
 */
export declare function getEndpoint(args: GetEndpointArgs, opts?: pulumi.InvokeOptions): Promise<GetEndpointResult>;
/**
 * A collection of arguments for invoking getEndpoint.
 */
export interface GetEndpointArgs {
    /**
     * Database endpoint identifier. Identifiers must contain from 1 to 255 alphanumeric characters or hyphens, begin with a letter, contain only ASCII letters, digits, and hyphens, not end with a hyphen, and not contain two consecutive hyphens.
     */
    endpointId: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getEndpoint.
 */
export interface GetEndpointResult {
    readonly certificateArn: string;
    readonly databaseName: string;
    readonly elasticsearchSettings: outputs.dms.GetEndpointElasticsearchSetting[];
    readonly endpointArn: string;
    readonly endpointId: string;
    readonly endpointType: string;
    readonly engineName: string;
    readonly extraConnectionAttributes: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly kafkaSettings: outputs.dms.GetEndpointKafkaSetting[];
    readonly kinesisSettings: outputs.dms.GetEndpointKinesisSetting[];
    readonly kmsKeyArn: string;
    readonly mongodbSettings: outputs.dms.GetEndpointMongodbSetting[];
    readonly mysqlSettings: outputs.dms.GetEndpointMysqlSetting[];
    readonly password: string;
    readonly port: number;
    readonly postgresSettings: outputs.dms.GetEndpointPostgresSetting[];
    readonly redisSettings: outputs.dms.GetEndpointRedisSetting[];
    readonly redshiftSettings: outputs.dms.GetEndpointRedshiftSetting[];
    readonly region: string;
    readonly s3Settings: outputs.dms.GetEndpointS3Setting[];
    readonly secretsManagerAccessRoleArn: string;
    readonly secretsManagerArn: string;
    readonly serverName: string;
    readonly serviceAccessRole: string;
    readonly sslMode: string;
    readonly tags: {
        [key: string]: string;
    };
    readonly username: string;
}
/**
 * Data source for managing an AWS DMS (Database Migration) Endpoint.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const test = aws.dms.getEndpoint({
 *     endpointId: "test_id",
 * });
 * ```
 */
export declare function getEndpointOutput(args: GetEndpointOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetEndpointResult>;
/**
 * A collection of arguments for invoking getEndpoint.
 */
export interface GetEndpointOutputArgs {
    /**
     * Database endpoint identifier. Identifiers must contain from 1 to 255 alphanumeric characters or hyphens, begin with a letter, contain only ASCII letters, digits, and hyphens, not end with a hyphen, and not contain two consecutive hyphens.
     */
    endpointId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
