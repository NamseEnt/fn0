import { ConfigMap, ConfigValue } from "./config";
import { EngineEvent } from "./events";
import { TagMap } from "./tag";
import { Deployment, PulumiFn, Workspace } from "./workspace";
/**
 * {@link Stack} is an isolated, independently configurable instance of a Pulumi
 * program. {@link Stack} exposes methods for the full Pulumi lifecycle
 * (up/preview/refresh/destroy), as well as managing configuration. Multiple
 * {@link Stacks} are commonly used to denote different phases of development
 * (such as development, staging and production) or feature branches (such as
 * feature-x-dev, jane-feature-x-dev).
 *
 * @alpha
 */
export declare class Stack {
    /**
     * The name identifying the stack.
     */
    name: string;
    /**
     * The {@link Workspace} the stack was created from.
     */
    readonly workspace: Workspace;
    private ready;
    /**
     * Creates a new stack using the given workspace, and stack name.
     * It fails if a stack with that name already exists
     *
     * @param name
     *  The name identifying the Stack.
     * @param workspace
     *  The Workspace the Stack was created from.
     */
    static create(name: string, workspace: Workspace): Promise<Stack>;
    /**
     * Selects stack using the given workspace and stack name. It returns an
     * error if the given stack does not exist.
     *
     * @param name
     *  The name identifying the Stack.
     * @param workspace
     *  The {@link Workspace} the stack will be created from.
     */
    static select(name: string, workspace: Workspace): Promise<Stack>;
    /**
     * Creates a new stack using the given workspace and stack name if the stack
     * does not already exist, or falls back to selecting the existing stack. If
     * the stack does not exist, it will be created and selected.
     *
     * @param name
     *  The name identifying the Stack.
     * @param workspace
     *  The {@link Workspace} the stack will be created from.
     */
    static createOrSelect(name: string, workspace: Workspace): Promise<Stack>;
    private constructor();
    private setupEventLog;
    private readLines;
    /**
     * Creates or updates the resources in a stack by executing the program in
     * the {@link Workspace.}
     *
     * @param opts
     *  Options to customize the behavior of the update.
     *
     * @see https://www.pulumi.com/docs/cli/commands/pulumi_up/
     */
    up(opts?: UpOptions): Promise<UpResult>;
    /**
     * Performs a dry-run update to a stack, returning pending changes.
     *
     * @param opts Options to customize the behavior of the preview.
     *
     * @see https://www.pulumi.com/docs/cli/commands/pulumi_preview/
     */
    preview(opts?: PreviewOptions): Promise<PreviewResult>;
    /**
     * Check the installed version of the Pulumi CLI supports inline programs for refresh and destroy operations.
     */
    private checkInlineSupport;
    /**
     * Compares the current stack’s resource state with the state known to exist
     * in the actual cloud provider. Any such changes are adopted into the
     * current stack.
     *
     * @param opts
     *  Options to customize the behavior of the refresh.
     */
    refresh(opts?: RefreshOptions): Promise<RefreshResult>;
    /**
     * Performs a dry-run refresh of the stack, returning pending changes.
     *
     * @param opts
     *  Options to customize the behavior of the refresh.
     */
    previewRefresh(opts?: RefreshOptions): Promise<PreviewResult>;
    /**
     * Deletes all resources in a stack. By default, this method will leave all
     * history and configuration intact. If `opts.remove` is set, the entire
     * stack and its configuration will also be deleted.
     *
     * @param opts
     *  Options to customize the behavior of the destroy.
     */
    destroy(opts?: DestroyOptions): Promise<DestroyResult>;
    /**
     * Performs a dry-run destroy of the stack, returning pending changes.
     *
     * @param opts
     *  Options to customize the behavior of the destroy.
     */
    previewDestroy(opts?: DestroyOptions): Promise<PreviewResult>;
    /**
     * Rename an existing stack
     */
    rename(options: RenameOptions): Promise<RenameResult>;
    /**
     * Import resources into the stack
     *
     * @param options Options to specify resources and customize the behavior of the import.
     */
    import(options: ImportOptions): Promise<ImportResult>;
    /**
     * Adds environments to the end of a stack's import list. Imported
     * environments are merged in order per the ESC merge rules. The list of
     * environments behaves as if it were the import list in an anonymous
     * environment.
     *
     * @param environments
     *  The names of the environments to add to the stack's configuration
     */
    addEnvironments(...environments: string[]): Promise<void>;
    /**
     * Returns the list of environments currently in the stack's import list.
     */
    listEnvironments(): Promise<string[]>;
    /**
     * Removes an environment from a stack's import list.
     *
     * @param environment
     *  The name of the environment to remove from the stack's configuration
     */
    removeEnvironment(environment: string): Promise<void>;
    /**
     * Returns the config value associated with the specified key.
     *
     * @param key
     *  The key to use for the config lookup
     * @param path
     *  The key contains a path to a property in a map or list to get
     */
    getConfig(key: string, path?: boolean): Promise<ConfigValue>;
    /**
     * Returns the full config map associated with the stack in the workspace.
     */
    getAllConfig(): Promise<ConfigMap>;
    /**
     * Sets a config key-value pair on the stack in the associated Workspace.
     *
     * @param key
     *  The key to set.
     * @param value
     *  The config value to set.
     * @param path
     *  The key contains a path to a property in a map or list to set.
     */
    setConfig(key: string, value: ConfigValue, path?: boolean): Promise<void>;
    /**
     * Sets all specified config values on the stack in the associated
     * workspace.
     *
     * @param config
     *  The map of config key-value pairs to set.
     * @param path
     *  The keys contain a path to a property in a map or list to set.
     */
    setAllConfig(config: ConfigMap, path?: boolean): Promise<void>;
    /**
     * Sets all config values from a JSON string for the stack in the associated workspace.
     * The JSON string should be in the format produced by "pulumi config --json".
     *
     * @param configJson
     *  A JSON string containing the configuration values to set
     */
    setAllConfigJson(configJson: string): Promise<void>;
    /**
     * Removes the specified config key from the stack in the associated workspace.
     *
     * @param key
     *  The config key to remove.
     * @param path
     *  The key contains a path to a property in a map or list to remove.
     */
    removeConfig(key: string, path?: boolean): Promise<void>;
    /**
     * Removes the specified config keys from the stack in the associated workspace.
     *
     * @param keys
     *  The config keys to remove.
     * @param path
     *  The keys contain a path to a property in a map or list to remove.
     */
    removeAllConfig(keys: string[], path?: boolean): Promise<void>;
    /**
     * Gets and sets the config map used with the last update.
     */
    refreshConfig(): Promise<ConfigMap>;
    /**
     * Returns the tag value associated with specified key.
     *
     * @param key The key to use for the tag lookup.
     */
    getTag(key: string): Promise<string>;
    /**
     * Sets a tag key-value pair on the stack in the associated workspace.
     *
     * @param key
     *  The tag key to set.
     * @param value
     *  The tag value to set.
     */
    setTag(key: string, value: string): Promise<void>;
    /**
     * Removes the specified tag key-value pair from the stack in the associated
     * workspace.
     *
     * @param key The tag key to remove.
     */
    removeTag(key: string): Promise<void>;
    /**
     * Returns the full tag map associated with the stack in the workspace.
     */
    listTags(): Promise<TagMap>;
    /**
     * Gets the current set of stack outputs from the last {@link Stack.up}.
     */
    outputs(): Promise<OutputMap>;
    /**
     * Returns a list summarizing all previous and current results from Stack
     * lifecycle operations (up/preview/refresh/destroy).
     */
    history(pageSize?: number, page?: number, showSecrets?: boolean): Promise<UpdateSummary[]>;
    info(showSecrets?: boolean): Promise<UpdateSummary | undefined>;
    /**
     * Stops a stack's currently running update. It returns an error if no
     * update is currently running. Note that this operation is _very
     * dangerous_, and may leave the stack in an inconsistent state if a
     * resource operation was pending when the update was canceled.
     */
    cancel(): Promise<void>;
    /**
     * Exports the deployment state of the stack. This can be combined with
     * {@link Stack.importStack} to edit a stack's state (such as recovery from
     * failed deployments).
     */
    exportStack(): Promise<Deployment>;
    /**
     * Imports the specified deployment state into a pre-existing stack. This
     * can be combined with {@link Stack.exportStack} to edit a stack's state
     * (such as recovery from failed deployments).
     *
     * @param state
     *  The stack state to import.
     */
    importStack(state: Deployment): Promise<void>;
    private runPulumiCmd;
    private get isRemote();
    private remoteArgs;
}
/**
 * Returns a stack name formatted with the greatest possible specificity:
 * `org/project/stack` or `user/project/stack` Using this format avoids
 * ambiguity in stack identity guards creating or selecting the wrong stack.
 *
 * Note: legacy DIY backends (local file, S3, Azure Blob) do not support
 * stack names in this format, and instead only use the stack name without an
 * org/user or project to qualify it.
 *
 * See: https://github.com/pulumi/pulumi/issues/2522
 *
 * Non-legacy DIY backends do support the `org/project/stack` format, but `org`
 * must be set to "organization".
 *
 * @param org
 *  The org (or user) that contains the Stack.
 * @param project
 *  The project that parents the Stack.
 * @param stack
 *  The name of the Stack.
 */
export declare function fullyQualifiedStackName(org: string, project: string, stack: string): string;
/**
 * A set of outputs, keyed by name, that might be returned by a Pulumi program
 * as part of a stack operation.
 */
export declare type OutputMap = {
    [key: string]: OutputValue;
};
/**
 * An output produced by a Pulumi program as part of a stack operation.
 */
export interface OutputValue {
    /**
     * The underlying output value.
     */
    value: any;
    /**
     * True if and only if the value represents a secret.
     */
    secret: boolean;
}
/**
 * A summary of a stack operation.
 */
export interface UpdateSummary {
    /**
     * The kind of operation to be executed/that was executed.
     */
    kind: UpdateKind;
    /**
     * The time at which the operation started.
     */
    startTime: Date;
    /**
     * An optional message associated with the operation.
     */
    message: string;
    /**
     * The environment supplied to the operation.
     */
    environment: {
        [key: string]: string;
    };
    /**
     * The configuration used for the operation.
     */
    config: ConfigMap;
    /**
     * The operation result.
     */
    result: UpdateResult;
    /**
     * The time at which the operation completed.
     */
    endTime: Date;
    /**
     * The version of the stack created by the operation.
     */
    version: number;
    /**
     * A raw JSON blob detailing the deployment.
     */
    Deployment?: RawJSON;
    /**
     * A summary of the changes yielded by the operation (e.g. 4 unchanged, 3
     * created, etc.).
     */
    resourceChanges?: OpMap;
}
/**
 * The kind of update that was performed on the stack.
 */
export declare type UpdateKind = "update" | "preview" | "refresh" | "rename" | "destroy" | "import";
/**
 * Represents the current status of a given update.
 */
export declare type UpdateResult = "not-started" | "in-progress" | "succeeded" | "failed";
/**
 * The granular CRUD operation performed on a particular resource during an update.
 */
export declare type OpType = "same" | "create" | "update" | "delete" | "replace" | "create-replacement" | "delete-replaced" | "read" | "read-replacement" | "refresh" | "discard" | "discard-replaced" | "remove-pending-replace" | "import" | "import-replacement";
/**
 * A map of operation types and their corresponding counts.
 */
export declare type OpMap = {
    [key in OpType]?: number;
};
/**
 * An unstructured JSON string used for back-compat with versioned APIs (such as Deployment).
 */
export declare type RawJSON = string;
/**
 * The deployment output from running a Pulumi program update.
 */
export interface UpResult {
    /**
     * The standard output from the update.
     */
    stdout: string;
    /**
     * The standard error output from the update.
     */
    stderr: string;
    /**
     * The outputs from the update.
     */
    outputs: OutputMap;
    /**
     * A summary of the update.
     */
    summary: UpdateSummary;
}
/**
 * Output from running a Pulumi program preview.
 */
export interface PreviewResult {
    /**
     * The standard output from the preview.
     */
    stdout: string;
    /**
     * The standard error output from the preview.
     */
    stderr: string;
    /**
     * A summary of the changes yielded by the operation (e.g. 4 unchanged, 3
     * created, etc.).
     */
    changeSummary: OpMap;
}
/**
 * Output from refreshing the resources in a given Stack.
 */
export interface RefreshResult {
    /**
     * The standard output from the refresh.
     */
    stdout: string;
    /**
     * The standard error output from the refresh.
     */
    stderr: string;
    /**
     * A summary of the refresh.
     */
    summary: UpdateSummary;
}
/**
 * Output from destroying all resources in a Stack.
 */
export interface DestroyResult {
    /**
     * The standard output from the destroy.
     */
    stdout: string;
    /**
     * The standard error output from the destroy.
     */
    stderr: string;
    /**
     * A summary of the destroy.
     */
    summary: UpdateSummary;
}
/**
 * Output from renaming the Stack.
 */
export interface RenameResult {
    /**
     * The standard output from the rename.
     */
    stdout: string;
    /**
     * The standard error output from the rename.
     */
    stderr: string;
    /**
     * A summary of the rename.
     */
    summary: UpdateSummary;
}
/**
 * The output from performing an import operation.
 */
export interface ImportResult {
    stdout: string;
    stderr: string;
    generatedCode: string;
    summary: UpdateSummary;
}
export interface GlobalOpts {
    /**
     * Colorize output.
     */
    color?: "always" | "never" | "raw" | "auto";
    /**
     * Flow log settings to child processes (like plugins)
     */
    logFlow?: boolean;
    /**
     * Enable verbose logging (e.g., v=3); anything >3 is very verbose.
     */
    logVerbosity?: number;
    /**
     * Log to stderr instead of to files.
     */
    logToStdErr?: boolean;
    /**
     * Emit tracing to the specified endpoint. Use the `file:` scheme to write tracing data to a local files.
     */
    tracing?: string;
    /**
     * Print detailed debugging output during resource operations.
     * */
    debug?: boolean;
    /**
     * Suppress display of stack outputs (in case they contain sensitive values).
     */
    suppressOutputs?: boolean;
    /**
     * Suppress display of periodic progress dots.
     */
    suppressProgress?: boolean;
    /**
     * Use the configuration values in the specified file rather than detecting the file name.
     */
    configFile?: string;
    /**
     * Save any creates seen during the preview into an import file to use with `pulumi import`.
     */
    importFile?: string;
}
/**
 * Options controlling the behavior of a Stack.up() operation.
 */
export interface UpOptions extends GlobalOpts {
    /**
     * Allow P resource operations to run in parallel at once (1 for no parallelism).
     */
    parallel?: number;
    /**
     * Optional message to associate with the operation.
     */
    message?: string;
    /**
     * Return an error if any changes occur during this operation.
     */
    expectNoChanges?: boolean;
    /**
     * Refresh the state of the stack's resources before this update.
     */
    refresh?: boolean;
    /**
     * Display the operation as a rich diff showing the overall change.
     */
    diff?: boolean;
    /**
     * Specify a set of resource URNs to replace.
     */
    replace?: string[];
    /**
     * Run one or more policy packs as part of this operation.
     */
    policyPacks?: string[];
    /**
     * A set of paths to JSON files containing configuration for the supplied `policyPacks`.
     */
    policyPackConfigs?: string[];
    /**
     * Specify a set of resource URNs to exclude from operations.
     */
    exclude?: string[];
    /**
     * Exclude dependents of targets specified with `exclude`.
     */
    excludeDependents?: boolean;
    /**
     * Specify a set of resource URNs to operate on. Other resources will not be updated.
     */
    target?: string[];
    /**
     * Operate on dependent targets discovered but not specified in `targets`.
     */
    targetDependents?: boolean;
    /**
     * A custom user agent to use when executing the operation.
     */
    userAgent?: string;
    /**
     * A callback to be executed when the operation produces stderr output.
     */
    onError?: (err: string) => void;
    /**
     * A callback to be executed when the operation produces stdout output.
     */
    onOutput?: (out: string) => void;
    /**
     * A callback to be executed when the operation yields an event.
     */
    onEvent?: (event: EngineEvent) => void;
    /**
     * An inline (in-process) Pulumi program to execute the operation against.
     */
    program?: PulumiFn;
    /**
     * Plan specifies the path to an update plan to use for the update.
     */
    plan?: string;
    /**
     * Include secrets in the UpSummary.
     */
    showSecrets?: boolean;
    /**
     * Continue the operation to completion even if errors occur.
     */
    continueOnError?: boolean;
    /**
     * Run the process under a debugger, and pause until a debugger is attached.
     */
    attachDebugger?: boolean;
    /**
     * A signal to abort an ongoing operation.
     */
    signal?: AbortSignal;
    /**
     * Run the program in the workspace to perform the refresh.
     */
    runProgram?: boolean;
}
/**
 * Options controlling the behavior of a Stack.preview() operation.
 */
export interface PreviewOptions extends GlobalOpts {
    /**
     * Allow P resource operations to run in parallel at once (1 for no parallelism).
     */
    parallel?: number;
    /**
     * Optional message to associate with the operation.
     */
    message?: string;
    /**
     * Return an error if any changes occur during this operation.
     */
    expectNoChanges?: boolean;
    /**
     * Refresh the state of the stack's resources against the cloud provider before running preview.
     */
    refresh?: boolean;
    /**
     * Display the operation as a rich diff showing the overall change.
     */
    diff?: boolean;
    /**
     * Specify a set of resource URNs to replace.
     */
    replace?: string[];
    /**
     * Run one or more policy packs as part of this operation.
     */
    policyPacks?: string[];
    /**
     * A set of paths to JSON files containing configuration for the supplied `policyPacks`.
     */
    policyPackConfigs?: string[];
    /**
     * Specify a set of resource URNs to exclude from operations.
     */
    exclude?: string[];
    /**
     * Exclude dependents of targets specified with `exclude`.
     */
    excludeDependents?: boolean;
    /**
     * Specify a set of resource URNs to operate on. Other resources will not be updated.
     */
    target?: string[];
    /**
     * Operate on dependent targets discovered but not specified in `targets`.
     */
    targetDependents?: boolean;
    /**
     * A custom user agent to use when executing the operation.
     */
    userAgent?: string;
    /**
     * An inline (in-process) Pulumi program to execute the operation against.
     */
    program?: PulumiFn;
    /**
     * A callback to be executed when the operation produces stdout output.
     */
    onOutput?: (out: string) => void;
    /**
     * A callback to be executed when the operation produces stderr output.
     */
    onError?: (err: string) => void;
    /**
     * A callback to be executed when the operation yields an event.
     */
    onEvent?: (event: EngineEvent) => void;
    /**
     * Plan specifies the path where the update plan should be saved.
     */
    plan?: string;
    /**
     * Run the process under a debugger, and pause until a debugger is attached.
     */
    attachDebugger?: boolean;
    /**
     * A signal to abort an ongoing operation.
     */
    signal?: AbortSignal;
    /**
     * Run the program in the workspace to perform the refresh.
     */
    runProgram?: boolean;
}
/**
 * Options controlling the behavior of a Stack.refresh() operation.
 */
export interface RefreshOptions extends GlobalOpts {
    /**
     * Allow P resource operations to run in parallel at once (1 for no parallelism).
     */
    parallel?: number;
    /**
     * Optional message to associate with the operation.
     */
    message?: string;
    /**
     * Only show a preview of the refresh, but don't perform the refresh itself.
     * @deprecated Use `previewRefresh` instead.
     */
    previewOnly?: boolean;
    /**
     * Return an error if any changes occur during this operation.
     */
    expectNoChanges?: boolean;
    /**
     * Clear all pending creates, dropping them from the state
     */
    clearPendingCreates?: boolean;
    /**
     * Specify a set of resource URNs to exclude from operations.
     */
    exclude?: string[];
    /**
     * Exclude dependents of targets specified with `exclude`.
     */
    excludeDependents?: boolean;
    /**
     * Specify a set of resource URNs to operate on. Other resources will not be updated.
     */
    target?: string[];
    /**
     * Operate on dependent targets discovered but not specified in `targets`.
     */
    targetDependents?: boolean;
    /**
     * A custom user agent to use when executing the operation.
     */
    userAgent?: string;
    /**
     * A callback to be executed when the operation produces stderr output.
     */
    onError?: (err: string) => void;
    /**
     * A callback to be executed when the operation produces stdout output.
     */
    onOutput?: (out: string) => void;
    /**
     * A callback to be executed when the operation yields an event.
     */
    onEvent?: (event: EngineEvent) => void;
    /**
     * Include secrets in the operation summary.
     */
    showSecrets?: boolean;
    /**
     * A signal to abort an ongoing operation.
     */
    signal?: AbortSignal;
    /**
     * Run the program in the workspace to perform the refresh.
     */
    runProgram?: boolean;
}
/**
 * Options controlling the behavior of a Stack.destroy() operation.
 */
export interface DestroyOptions extends GlobalOpts {
    /**
     * Allow P resource operations to run in parallel at once (1 for no parallelism).
     */
    parallel?: number;
    /**
     * Optional message to associate with the operation.
     */
    message?: string;
    /**
     * Refresh the state of the stack's resources against the cloud provider before running destroy.
     */
    refresh?: boolean;
    /**
     * Specify a set of resource URNs to exclude from operations.
     */
    exclude?: string[];
    /**
     * Exclude dependents of targets specified with `exclude`.
     */
    excludeDependents?: boolean;
    /**
     * Specify a set of resource URNs to operate on. Other resources will not be updated.
     */
    target?: string[];
    /**
     * Operate on dependent targets discovered but not specified in `targets`.
     */
    targetDependents?: boolean;
    /**
     * A custom user agent to use when executing the operation.
     */
    userAgent?: string;
    /**
     * A callback to be executed when the operation produces stderr output.
     */
    onError?: (err: string) => void;
    /**
     * A callback to be executed when the operation produces stdout output.
     */
    onOutput?: (out: string) => void;
    /**
     * A callback to be executed when the operation yields an event.
     */
    onEvent?: (event: EngineEvent) => void;
    /**
     * Include secrets in the operation summary.
     */
    showSecrets?: boolean;
    /**
     * Do not destroy protected resources.
     */
    excludeProtected?: boolean;
    /**
     * Continue the operation to completion even if errors occur.
     */
    continueOnError?: boolean;
    /**
     * Only show a preview of the destroy, but don't perform the destroy itself.
     * @deprecated Use `previewDestroy` instead.
     */
    previewOnly?: boolean;
    /**
     * Remove the stack and its configuration after all resources in the stack have been deleted.
     */
    remove?: boolean;
    /**
     * A signal to abort an ongoing operation.
     */
    signal?: AbortSignal;
    /**
     * Run the program in the workspace to perform the destroy.
     */
    runProgram?: boolean;
}
/**
 * Options controlling the behavior of a Stack.rename() operation.
 */
export interface RenameOptions extends GlobalOpts {
    /**
     * The new name for the stack.
     */
    stackName: string;
    /**
     * A callback to be executed when the operation produces stderr output.
     */
    onError?: (err: string) => void;
    /**
     * A callback to be executed when the operation produces stdout output.
     */
    onOutput?: (out: string) => void;
    /**
     * Include secrets in the UpSummary.
     */
    showSecrets?: boolean;
    /**
     * A signal to abort an ongoing operation.
     */
    signal?: AbortSignal;
}
export interface ImportResource {
    /**
     * The type of the resource to import
     */
    type: string;
    /**
     * The name of the resource to import
     */
    name: string;
    /**
     * The ID of the resource to import. The format of the ID is specific to the resource type.
     */
    id?: string;
    parent?: string;
    provider?: string;
    version?: string;
    pluginDownloadUrl?: string;
    logicalName?: string;
    properties?: string[];
    component?: boolean;
    remote?: boolean;
}
/**
 * Options controlling the behavior of a Stack.import() operation.
 */
export interface ImportOptions extends GlobalOpts {
    /**
     * The resource definitions to import into the stack
     */
    resources?: ImportResource[];
    /**
     * The name table maps language names to parent and provider URNs. These names are
     * used in the generated definitions, and should match the corresponding declarations
     * in the source program. This table is required if any parents or providers are
     * specified by the resources to import.
     */
    nameTable?: {
        [key: string]: string;
    };
    /**
     * Allow resources to be imported with protection from deletion enabled. Set to true by default.
     */
    protect?: boolean;
    /**
     * Generate resource declaration code for the imported resources. Set to true by default.
     */
    generateCode?: boolean;
    /**
     * Specify the name of a converter to import resources from.
     */
    converter?: string;
    /**
     * Additional arguments to pass to the converter, if the user specified one.
     */
    converterArgs?: string[];
    /**
     * Optional message to associate with the import operation
     */
    message?: string;
    /**
     * Include secrets in the import result summary
     */
    showSecrets?: boolean;
    onOutput?: (out: string) => void;
}
