import * as outputs from "../types/output";
/**
 * PEM-encoded content of Docker host CA certificate
 */
export declare const caMaterial: string | undefined;
/**
 * PEM-encoded content of Docker client certificate
 */
export declare const certMaterial: string | undefined;
/**
 * Path to directory with Docker TLS config
 */
export declare const certPath: string | undefined;
/**
 * The name of the Docker context to use. Can also be set via `DOCKER_CONTEXT` environment variable. Overrides the `host` if set.
 */
export declare const context: string | undefined;
/**
 * If set to `true`, the provider will not check if the Docker daemon is running. This is useful for resources/data_sourcess that do not require a running Docker daemon, such as the data source `docker.RegistryImage`.
 */
export declare const disableDockerDaemonCheck: boolean | undefined;
/**
 * The Docker daemon address
 */
export declare const host: string | undefined;
/**
 * PEM-encoded content of Docker client private key
 */
export declare const keyMaterial: string | undefined;
export declare const registryAuth: outputs.config.RegistryAuth[] | undefined;
/**
 * Additional SSH option flags to be appended when using `ssh://` protocol
 */
export declare const sshOpts: string[] | undefined;
