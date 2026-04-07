import * as pulumi from "@pulumi/pulumi";
import * as outputs from "../types/output";
/**
 * Lists the provisioning artifacts for the specified product.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.servicecatalog.getProvisioningArtifacts({
 *     productId: "prod-yakog5pdriver",
 * });
 * ```
 */
export declare function getProvisioningArtifacts(args: GetProvisioningArtifactsArgs, opts?: pulumi.InvokeOptions): Promise<GetProvisioningArtifactsResult>;
/**
 * A collection of arguments for invoking getProvisioningArtifacts.
 */
export interface GetProvisioningArtifactsArgs {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: string;
    /**
     * Product identifier.
     *
     * The following arguments are optional:
     */
    productId: string;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: string;
}
/**
 * A collection of values returned by getProvisioningArtifacts.
 */
export interface GetProvisioningArtifactsResult {
    readonly acceptLanguage?: string;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    readonly productId: string;
    /**
     * List with information about the provisioning artifacts. See details below.
     */
    readonly provisioningArtifactDetails: outputs.servicecatalog.GetProvisioningArtifactsProvisioningArtifactDetail[];
    readonly region: string;
}
/**
 * Lists the provisioning artifacts for the specified product.
 *
 * ## Example Usage
 *
 * ### Basic Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as aws from "@pulumi/aws";
 *
 * const example = aws.servicecatalog.getProvisioningArtifacts({
 *     productId: "prod-yakog5pdriver",
 * });
 * ```
 */
export declare function getProvisioningArtifactsOutput(args: GetProvisioningArtifactsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetProvisioningArtifactsResult>;
/**
 * A collection of arguments for invoking getProvisioningArtifacts.
 */
export interface GetProvisioningArtifactsOutputArgs {
    /**
     * Language code. Valid values: `en` (English), `jp` (Japanese), `zh` (Chinese). Default value is `en`.
     */
    acceptLanguage?: pulumi.Input<string>;
    /**
     * Product identifier.
     *
     * The following arguments are optional:
     */
    productId: pulumi.Input<string>;
    /**
     * Region where this resource will be [managed](https://docs.aws.amazon.com/general/latest/gr/rande.html#regional-endpoints). Defaults to the Region set in the provider configuration.
     */
    region?: pulumi.Input<string>;
}
