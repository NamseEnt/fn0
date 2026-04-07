import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Data source for managing an AWS VPC Lattice Service.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.vpclattice.getService({
 *     name: "example",
 * });
 * ```
 */
export declare function getService(args?: GetServiceArgs, opts?: pulumi.InvokeOptions): Promise<GetServiceResult>;
/**
 * A collection of arguments for invoking getService.
 */
export interface GetServiceArgs {
    /**
     * Service name.
     */
    name?: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
    /**
     * ID or Amazon Resource Name (ARN) of the service.
     */
    serviceIdentifier?: string;
    /**
     * List of tags associated with the service.
     */
    tags?: {
        [key: string]: string;
    };
}
/**
 * A collection of values returned by getService.
 */
export interface GetServiceResult {
    /**
     * ARN of the service.
     */
    readonly arn: string;
    /**
     * Type of IAM policy. Either `NONE` or `AWS_IAM`.
     */
    readonly authType: string;
    /**
     * Amazon Resource Name (ARN) of the certificate.
     */
    readonly certificateArn: string;
    /**
     * Custom domain name of the service.
     */
    readonly customDomainName: string;
    /**
     * List of objects with DNS names.
     */
    readonly dnsEntries: outputs.vpclattice.GetServiceDnsEntry[];
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly name: string;
    readonly region: string;
    readonly serviceIdentifier: string;
    /**
     * Status of the service.
     */
    readonly status: string;
    /**
     * List of tags associated with the service.
     */
    readonly tags: {
        [key: string]: string;
    };
}
/**
 * Data source for managing an AWS VPC Lattice Service.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.vpclattice.getService({
 *     name: "example",
 * });
 * ```
 */
export declare function getServiceOutput(args?: GetServiceOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetServiceResult>;
/**
 * A collection of arguments for invoking getService.
 */
export interface GetServiceOutputArgs {
    /**
     * Service name.
     */
    name?: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
    /**
     * ID or Amazon Resource Name (ARN) of the service.
     */
    serviceIdentifier?: pulumi.Input<string>;
    /**
     * List of tags associated with the service.
     */
    tags?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
}
