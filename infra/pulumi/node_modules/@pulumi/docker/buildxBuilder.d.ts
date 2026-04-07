import * as pulumi from "@pulumi/pulumi";
import * as inputs from "./types/input";
import * as outputs from "./types/output";
/**
 * Manages a Docker Buildx builder instance. This resource allows you to create a  buildx builder with various configurations such as driver, nodes, and platform settings. Please see https://github.com/docker/buildx/blob/master/docs/reference/buildx_create.md for more documentation
 */
export declare class BuildxBuilder extends pulumi.CustomResource {
    /**
     * Get an existing BuildxBuilder resource's state with the given name, ID, and optional extra
     * properties used to qualify the lookup.
     *
     * @param name The _unique_ name of the resulting resource.
     * @param id The _unique_ provider ID of the resource to lookup.
     * @param state Any extra arguments used during the lookup.
     * @param opts Optional settings to control the behavior of the CustomResource.
     */
    static get(name: string, id: pulumi.Input<pulumi.ID>, state?: BuildxBuilderState, opts?: pulumi.CustomResourceOptions): BuildxBuilder;
    /**
     * Returns true if the given object is an instance of BuildxBuilder.  This is designed to work even
     * when multiple copies of the Pulumi SDK have been loaded into the same process.
     */
    static isInstance(obj: any): obj is BuildxBuilder;
    /**
     * Append a node to builder instead of changing it
     */
    readonly append: pulumi.Output<boolean | undefined>;
    /**
     * Automatically boot the builder after creation. Defaults to `false`
     */
    readonly bootstrap: pulumi.Output<boolean | undefined>;
    /**
     * BuildKit daemon config file
     */
    readonly buildkitConfig: pulumi.Output<string | undefined>;
    /**
     * BuildKit flags to set for the builder.
     */
    readonly buildkitFlags: pulumi.Output<string | undefined>;
    /**
     * Configuration block for the Docker-Container driver.
     */
    readonly dockerContainer: pulumi.Output<outputs.BuildxBuilderDockerContainer | undefined>;
    /**
     * The driver to use for the Buildx builder (e.g., docker-container, kubernetes).
     */
    readonly driver: pulumi.Output<string | undefined>;
    /**
     * Additional options for the Buildx driver in the form of `key=value,...`. These options are driver-specific.
     */
    readonly driverOptions: pulumi.Output<{
        [key: string]: string;
    } | undefined>;
    /**
     * The endpoint or context to use for the Buildx builder, where context is the name of a context from docker context ls and endpoint is the address for Docker socket (eg. DOCKER_HOST value). By default, the current Docker configuration is used for determining the context/endpoint value.
     */
    readonly endpoint: pulumi.Output<string | undefined>;
    /**
     * Configuration block for the Kubernetes driver.
     */
    readonly kubernetes: pulumi.Output<outputs.BuildxBuilderKubernetes | undefined>;
    /**
     * The name of the Buildx builder. IF not specified, a random name will be generated.
     */
    readonly name: pulumi.Output<string>;
    /**
     * Create/modify node with given name
     */
    readonly node: pulumi.Output<string | undefined>;
    /**
     * Fixed platforms for current node
     */
    readonly platforms: pulumi.Output<string[] | undefined>;
    /**
     * Configuration block for the Remote driver.
     */
    readonly remote: pulumi.Output<outputs.BuildxBuilderRemote | undefined>;
    /**
     * Set the current builder instance as the default for the current context.
     */
    readonly use: pulumi.Output<boolean | undefined>;
    /**
     * Create a BuildxBuilder resource with the given unique name, arguments, and options.
     *
     * @param name The _unique_ name of the resource.
     * @param args The arguments to use to populate this resource's properties.
     * @param opts A bag of options that control this resource's behavior.
     */
    constructor(name: string, args?: BuildxBuilderArgs, opts?: pulumi.CustomResourceOptions);
}
/**
 * Input properties used for looking up and filtering BuildxBuilder resources.
 */
export interface BuildxBuilderState {
    /**
     * Append a node to builder instead of changing it
     */
    append?: pulumi.Input<boolean>;
    /**
     * Automatically boot the builder after creation. Defaults to `false`
     */
    bootstrap?: pulumi.Input<boolean>;
    /**
     * BuildKit daemon config file
     */
    buildkitConfig?: pulumi.Input<string>;
    /**
     * BuildKit flags to set for the builder.
     */
    buildkitFlags?: pulumi.Input<string>;
    /**
     * Configuration block for the Docker-Container driver.
     */
    dockerContainer?: pulumi.Input<inputs.BuildxBuilderDockerContainer>;
    /**
     * The driver to use for the Buildx builder (e.g., docker-container, kubernetes).
     */
    driver?: pulumi.Input<string>;
    /**
     * Additional options for the Buildx driver in the form of `key=value,...`. These options are driver-specific.
     */
    driverOptions?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The endpoint or context to use for the Buildx builder, where context is the name of a context from docker context ls and endpoint is the address for Docker socket (eg. DOCKER_HOST value). By default, the current Docker configuration is used for determining the context/endpoint value.
     */
    endpoint?: pulumi.Input<string>;
    /**
     * Configuration block for the Kubernetes driver.
     */
    kubernetes?: pulumi.Input<inputs.BuildxBuilderKubernetes>;
    /**
     * The name of the Buildx builder. IF not specified, a random name will be generated.
     */
    name?: pulumi.Input<string>;
    /**
     * Create/modify node with given name
     */
    node?: pulumi.Input<string>;
    /**
     * Fixed platforms for current node
     */
    platforms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Configuration block for the Remote driver.
     */
    remote?: pulumi.Input<inputs.BuildxBuilderRemote>;
    /**
     * Set the current builder instance as the default for the current context.
     */
    use?: pulumi.Input<boolean>;
}
/**
 * The set of arguments for constructing a BuildxBuilder resource.
 */
export interface BuildxBuilderArgs {
    /**
     * Append a node to builder instead of changing it
     */
    append?: pulumi.Input<boolean>;
    /**
     * Automatically boot the builder after creation. Defaults to `false`
     */
    bootstrap?: pulumi.Input<boolean>;
    /**
     * BuildKit daemon config file
     */
    buildkitConfig?: pulumi.Input<string>;
    /**
     * BuildKit flags to set for the builder.
     */
    buildkitFlags?: pulumi.Input<string>;
    /**
     * Configuration block for the Docker-Container driver.
     */
    dockerContainer?: pulumi.Input<inputs.BuildxBuilderDockerContainer>;
    /**
     * The driver to use for the Buildx builder (e.g., docker-container, kubernetes).
     */
    driver?: pulumi.Input<string>;
    /**
     * Additional options for the Buildx driver in the form of `key=value,...`. These options are driver-specific.
     */
    driverOptions?: pulumi.Input<{
        [key: string]: pulumi.Input<string>;
    }>;
    /**
     * The endpoint or context to use for the Buildx builder, where context is the name of a context from docker context ls and endpoint is the address for Docker socket (eg. DOCKER_HOST value). By default, the current Docker configuration is used for determining the context/endpoint value.
     */
    endpoint?: pulumi.Input<string>;
    /**
     * Configuration block for the Kubernetes driver.
     */
    kubernetes?: pulumi.Input<inputs.BuildxBuilderKubernetes>;
    /**
     * The name of the Buildx builder. IF not specified, a random name will be generated.
     */
    name?: pulumi.Input<string>;
    /**
     * Create/modify node with given name
     */
    node?: pulumi.Input<string>;
    /**
     * Fixed platforms for current node
     */
    platforms?: pulumi.Input<pulumi.Input<string>[]>;
    /**
     * Configuration block for the Remote driver.
     */
    remote?: pulumi.Input<inputs.BuildxBuilderRemote>;
    /**
     * Set the current builder instance as the default for the current context.
     */
    use?: pulumi.Input<boolean>;
}
