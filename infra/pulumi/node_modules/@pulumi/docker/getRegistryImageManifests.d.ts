import * as pulumi from "@pulumi/pulumi";
import * as inputs from "./types/input";
import * as outputs from "./types/output";
/**
 * Reads the image metadata for each manifest in a Docker multi-arch image from a Docker Registry.
 */
export declare function getRegistryImageManifests(args: GetRegistryImageManifestsArgs, opts?: pulumi.InvokeOptions): Promise<GetRegistryImageManifestsResult>;
/**
 * A collection of arguments for invoking getRegistryImageManifests.
 */
export interface GetRegistryImageManifestsArgs {
    /**
     * Authentication configuration for the Docker registry. It is only used for this resource.
     */
    authConfig?: inputs.GetRegistryImageManifestsAuthConfig;
    /**
     * If `true`, the verification of TLS certificates of the server/registry is disabled. Defaults to `false`
     */
    insecureSkipVerify?: boolean;
    /**
     * The name of the Docker image, including any tags. e.g. `alpine:latest`
     */
    name: string;
}
/**
 * A collection of values returned by getRegistryImageManifests.
 */
export interface GetRegistryImageManifestsResult {
    /**
     * Authentication configuration for the Docker registry. It is only used for this resource.
     */
    readonly authConfig?: outputs.GetRegistryImageManifestsAuthConfig;
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * If `true`, the verification of TLS certificates of the server/registry is disabled. Defaults to `false`
     */
    readonly insecureSkipVerify?: boolean;
    /**
     * The metadata for each manifest in the image
     */
    readonly manifests: outputs.GetRegistryImageManifestsManifest[];
    /**
     * The name of the Docker image, including any tags. e.g. `alpine:latest`
     */
    readonly name: string;
}
/**
 * Reads the image metadata for each manifest in a Docker multi-arch image from a Docker Registry.
 */
export declare function getRegistryImageManifestsOutput(args: GetRegistryImageManifestsOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetRegistryImageManifestsResult>;
/**
 * A collection of arguments for invoking getRegistryImageManifests.
 */
export interface GetRegistryImageManifestsOutputArgs {
    /**
     * Authentication configuration for the Docker registry. It is only used for this resource.
     */
    authConfig?: pulumi.Input<inputs.GetRegistryImageManifestsAuthConfigArgs>;
    /**
     * If `true`, the verification of TLS certificates of the server/registry is disabled. Defaults to `false`
     */
    insecureSkipVerify?: pulumi.Input<boolean>;
    /**
     * The name of the Docker image, including any tags. e.g. `alpine:latest`
     */
    name: pulumi.Input<string>;
}
