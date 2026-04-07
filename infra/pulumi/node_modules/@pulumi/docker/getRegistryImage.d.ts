import * as pulumi from "@pulumi/pulumi";
/**
 * Reads the image metadata from a Docker Registry. Used in conjunction with the docker.RemoteImage resource to keep an image up to date on the latest available version of the tag.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as docker from "@pulumi/docker";
 *
 * const ubuntu = docker.getRegistryImage({
 *     name: "ubuntu:precise",
 * });
 * const ubuntuRemoteImage = new docker.RemoteImage("ubuntu", {
 *     name: ubuntu.then(ubuntu => ubuntu.name),
 *     pullTriggers: [ubuntu.then(ubuntu => ubuntu.sha256Digest)],
 * });
 * ```
 */
export declare function getRegistryImage(args: GetRegistryImageArgs, opts?: pulumi.InvokeOptions): Promise<GetRegistryImageResult>;
/**
 * A collection of arguments for invoking getRegistryImage.
 */
export interface GetRegistryImageArgs {
    /**
     * If `true`, the verification of TLS certificates of the server/registry is disabled. Defaults to `false`
     */
    insecureSkipVerify?: boolean;
    /**
     * The name of the Docker image, including any tags. e.g. `alpine:latest`. You can also specify a digest, e.g. `nginx:1.28.0@sha256:eaa7e36decc3421fc04478c586dfea0d931cebe47d5bc0b15d758a32ba51126f`.
     */
    name: string;
}
/**
 * A collection of values returned by getRegistryImage.
 */
export interface GetRegistryImageResult {
    /**
     * The provider-assigned unique ID for this managed resource.
     */
    readonly id: string;
    /**
     * If `true`, the verification of TLS certificates of the server/registry is disabled. Defaults to `false`
     */
    readonly insecureSkipVerify?: boolean;
    /**
     * The name of the Docker image, including any tags. e.g. `alpine:latest`. You can also specify a digest, e.g. `nginx:1.28.0@sha256:eaa7e36decc3421fc04478c586dfea0d931cebe47d5bc0b15d758a32ba51126f`.
     */
    readonly name: string;
    /**
     * The content digest of the image, as stored in the registry.
     */
    readonly sha256Digest: string;
}
/**
 * Reads the image metadata from a Docker Registry. Used in conjunction with the docker.RemoteImage resource to keep an image up to date on the latest available version of the tag.
 *
 * ## Example Usage
 *
 * ```typescript
 * import * as pulumi from "@pulumi/pulumi";
 * import * as docker from "@pulumi/docker";
 *
 * const ubuntu = docker.getRegistryImage({
 *     name: "ubuntu:precise",
 * });
 * const ubuntuRemoteImage = new docker.RemoteImage("ubuntu", {
 *     name: ubuntu.then(ubuntu => ubuntu.name),
 *     pullTriggers: [ubuntu.then(ubuntu => ubuntu.sha256Digest)],
 * });
 * ```
 */
export declare function getRegistryImageOutput(args: GetRegistryImageOutputArgs, opts?: pulumi.InvokeOutputOptions): pulumi.Output<GetRegistryImageResult>;
/**
 * A collection of arguments for invoking getRegistryImage.
 */
export interface GetRegistryImageOutputArgs {
    /**
     * If `true`, the verification of TLS certificates of the server/registry is disabled. Defaults to `false`
     */
    insecureSkipVerify?: pulumi.Input<boolean>;
    /**
     * The name of the Docker image, including any tags. e.g. `alpine:latest`. You can also specify a digest, e.g. `nginx:1.28.0@sha256:eaa7e36decc3421fc04478c586dfea0d931cebe47d5bc0b15d758a32ba51126f`.
     */
    name: pulumi.Input<string>;
}
