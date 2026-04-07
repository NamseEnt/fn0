import { PulumiCommand } from "./cmd";
import { ConfigMap, ConfigValue } from "./config";
import { ProjectSettings } from "./projectSettings";
import { ExecutorImage } from "./remoteWorkspace";
import { OutputMap, Stack } from "./stack";
import { StackSettings } from "./stackSettings";
import { TagMap } from "./tag";
import { Deployment, PluginInfo, PulumiFn, StackSummary, WhoAmIResult, Workspace } from "./workspace";
/**
 * {@link LocalWorkspace} is a default implementation of the {@link Workspace} interface.
 *
 * A {@link Workspace} is the execution context containing a single Pulumi
 * project, a program, and multiple stacks. Workspaces are used to manage the
 * execution environment, providing various utilities such as plugin
 * installation, environment configuration (`$PULUMI_HOME`), and creation,
 * deletion, and listing of Stacks.
 *
 * {@link LocalWorkspace} relies on `Pulumi.yaml` and `Pulumi.<stack>.yaml` as
 * the intermediate format for Project and Stack settings. Modifying the
 * workspace's {@link ProjectSettings} will alter the workspace's `Pulumi.yaml`
 * file, and setting config on a Stack will modify the relevant
 * `Pulumi.<stack>.yaml` file. This is identical to the behavior of Pulumi CLI
 * driven workspaces.
 *
 * @alpha
 */
export declare class LocalWorkspace implements Workspace {
    /**
     * The working directory to run Pulumi CLI commands in.
     */
    readonly workDir: string;
    /**
     * The directory override for CLI metadata if set. This customizes the
     * location of `$PULUMI_HOME` where metadata is stored and plugins are
     * installed.
     */
    readonly pulumiHome?: string;
    /**
     * The secrets provider to use for encryption and decryption of stack secrets.
     * See: https://www.pulumi.com/docs/intro/concepts/secrets/#available-encryption-providers
     */
    readonly secretsProvider?: string;
    private _pulumiCommand?;
    /**
     * The underlying Pulumi CLI.
     */
    get pulumiCommand(): PulumiCommand;
    /**
     * The image to use for the remote Pulumi operation.
     */
    remoteExecutorImage?: ExecutorImage;
    /**
     * The inline program {@link PulumiFn} to be used for preview/update
     * operations if any. If none is specified, the stack will refer to
     * {@link ProjectSettings} for this information.
     */
    program?: PulumiFn;
    /**
     * Environment values scoped to the current workspace. These will be supplied to every Pulumi command.
     */
    envVars: {
        [key: string]: string;
    };
    private _pulumiVersion?;
    /**
     * The version of the underlying Pulumi CLI/engine.
     *
     * @returns A string representation of the version, if available. `null` otherwise.
     */
    get pulumiVersion(): string;
    private ready;
    /**
     * True if the workspace is a remote workspace.
     */
    private remote?;
    /**
     * Remote Git source info.
     */
    private remoteGitProgramArgs?;
    /**
     * An optional list of arbitrary commands to run before the remote Pulumi operation is invoked.
     */
    private remotePreRunCommands?;
    /**
     * The environment variables to pass along when running remote Pulumi operations.
     */
    private remoteEnvVars?;
    /**
     * Whether to skip the default dependency installation step.
     */
    private remoteSkipInstallDependencies?;
    /**
     * Whether to inherit the deployment settings set on the stack.
     */
    private remoteInheritSettings?;
    /**
     * Creates a workspace using the specified options. Used for maximal control and customization
     * of the underlying environment before any stacks are created or selected.
     *
     * @param opts Options used to configure the Workspace
     */
    static create(opts: LocalWorkspaceOptions): Promise<LocalWorkspace>;
    /**
     * Creates a {@link Stack} with a {@link LocalWorkspace} utilizing the local
     * Pulumi CLI program from the specified working directory. This is a way to
     * create drivers on top of pre-existing Pulumi programs. This workspace
     * will pick up any available settings files (`Pulumi.yaml`,
     * `Pulumi.<stack>.yaml`).
     *
     * @param args
     *  A set of arguments to initialize a stack with a pre-configured Pulumi
     *  CLI program that already exists on disk.
     *
     * @param opts
     *  Additional customizations to be applied to the Workspace.
     */
    static createStack(args: LocalProgramArgs, opts?: LocalWorkspaceOptions): Promise<Stack>;
    /**
     * Creates a {@link Stack} with a {@link LocalWorkspace} utilizing the
     * specified inline (in process) Pulumi program. This program is fully
     * debuggable and runs in process. If no Project option is specified,
     * default project settings will be created on behalf of the user.
     * Similarly, unless a `workDir` option is specified, the working directory
     * will default to a new temporary directory provided by the OS.
     *
     * @param args
     *  A set of arguments to initialize a stack with and an inline
     *  {@link PulumiFn} program that runs in process.
     * @param opts
     *  Additional customizations to be applied to the Workspace.
     */
    static createStack(args: InlineProgramArgs, opts?: LocalWorkspaceOptions): Promise<Stack>;
    /**
     * Selects a {@link Stack} with a {@link LocalWorkspace} utilizing the local
     * Pulumi CLI program from the specified working directory. This is a way to
     * create drivers on top of pre-existing Pulumi programs. This Workspace
     * will pick up any available Settings files (`Pulumi.yaml`,
     * `Pulumi.<stack>.yaml`).
     *
     * @param args
     *  A set of arguments to initialize a stack with a pre-configured Pulumi
     *  CLI program that already exists on disk.
     * @param opts
     *  Additional customizations to be applied to the Workspace.
     */
    static selectStack(args: LocalProgramArgs, opts?: LocalWorkspaceOptions): Promise<Stack>;
    /**
     * Selects an existing {@link Stack} with a {@link LocalWorkspace} utilizing
     * the specified inline (in process) Pulumi program. This program is fully
     * debuggable and runs in process. If no Project option is specified,
     * default project settings will be created on behalf of the user.
     * Similarly, unless a `workDir` option is specified, the working directory
     * will default to a new temporary directory provided by the OS.
     *
     * @param args
     *  A set of arguments to initialize a Stack with and inline `PulumiFn`
     *  program that runs in process.
     * @param opts
     *  Additional customizations to be applied to the Workspace.
     */
    static selectStack(args: InlineProgramArgs, opts?: LocalWorkspaceOptions): Promise<Stack>;
    /**
     * Creates or selects an existing {@link Stack} with a {@link LocalWorkspace}
     * utilizing the specified inline (in process) Pulumi CLI program. This
     * program is fully debuggable and runs in process. If no project is
     * specified, default project settings will be created on behalf of the
     * user. Similarly, unless a `workDir` option is specified, the working
     * directory will default to a new temporary directory provided by the OS.
     *
     * @param args
     *  A set of arguments to initialize a Stack with a pre-configured Pulumi
     *  CLI program that already exists on disk.
     * @param opts
     *  Additional customizations to be applied to the Workspace.
     */
    static createOrSelectStack(args: LocalProgramArgs, opts?: LocalWorkspaceOptions): Promise<Stack>;
    /**
     * Creates or selects an existing {@link Stack} with a {@link
     * LocalWorkspace} utilizing the specified inline Pulumi CLI program. This
     * program is fully debuggable and runs in process. If no Project option is
     * specified, default project settings will be created on behalf of the
     * user. Similarly, unless a `workDir` option is specified, the working
     * directory will default to a new temporary directory provided by the OS.
     *
     * @param args
     *  A set of arguments to initialize a Stack with and inline `PulumiFn`
     *  program that runs in process.
     * @param opts
     *  Additional customizations to be applied to the Workspace.
     */
    static createOrSelectStack(args: InlineProgramArgs, opts?: LocalWorkspaceOptions): Promise<Stack>;
    private static localSourceStackHelper;
    private static inlineSourceStackHelper;
    /**
     * Constructs a new {@link LocalWorkspace}.
     */
    private constructor();
    /**
     * Returns the settings object for the current project if any
     * {@link LocalWorkspace} reads settings from the `Pulumi.yaml`
     * in the workspace. A workspace can contain only a single project at a
     * time.
     */
    projectSettings(): Promise<ProjectSettings>;
    /**
     * Overwrites the settings object in the current project. There can only be
     * a single project per workspace. Fails if new project name does not match
     * old. {@link LocalWorkspace} writes this value to a `Pulumi.yaml` file in
     * `Workspace.WorkDir()`.
     *
     * @param settings
     *  The settings object to save to the Workspace.
     */
    saveProjectSettings(settings: ProjectSettings): Promise<void>;
    /**
     * Returns the settings object for the stack matching the specified stack
     * name if any. {@link LocalWorkspace} reads this from a
     * `Pulumi.<stack>.yaml` file in `Workspace.WorkDir()`.
     *
     * @param stackName
     *  The stack to retrieve settings from.
     */
    stackSettings(stackName: string): Promise<StackSettings>;
    /**
     * Overwrites the settings object for the stack matching the specified stack
     * name. {@link LocalWorkspace} writes this value to a `Pulumi.<stack>.yaml`
     * file in `Workspace.WorkDir()`
     *
     * @param stackName
     *  The stack to operate on.
     * @param settings
     *  The settings object to save.
     */
    saveStackSettings(stackName: string, settings: StackSettings): Promise<void>;
    /**
     * Creates and sets a new stack with the stack name, failing if one already
     * exists.
     *
     * @param stackName The stack to create.
     */
    createStack(stackName: string): Promise<void>;
    /**
     * Selects and sets an existing stack matching the stack name, failing if
     * none exists.
     *
     * @param stackName The stack to select.
     */
    selectStack(stackName: string): Promise<void>;
    /**
     * Deletes the stack and all associated configuration and history.
     *
     * @param stackName The stack to remove
     */
    removeStack(stackName: string, opts?: RemoveOptions): Promise<void>;
    /**
     * Adds environments to the end of a stack's import list. Imported
     * environments are merged in order per the ESC merge rules. The list of
     * environments behaves as if it were the import list in an anonymous
     * environment.
     *
     * @param stackName
     *  The stack to operate on
     * @param environments
     *  The names of the environments to add to the stack's configuration
     */
    addEnvironments(stackName: string, ...environments: string[]): Promise<void>;
    /**
     * Returns the list of environments associated with the specified stack name.
     *
     * @param stackName The stack to operate on
     */
    listEnvironments(stackName: string): Promise<string[]>;
    /**
     * Removes an environment from a stack's import list.
     *
     * @param stackName
     *  The stack to operate on
     * @param environment
     *  The name of the environment to remove from the stack's configuration
     */
    removeEnvironment(stackName: string, environment: string): Promise<void>;
    /**
     * Returns the value associated with the specified stack name and key,
     * scoped to the current workspace. {@link LocalWorkspace} reads this config
     * from the matching `Pulumi.stack.yaml` file.
     *
     * @param stackName
     *  The stack to read config from
     * @param key
     *  The key to use for the config lookup
     * @param path
     *  The key contains a path to a property in a map or list to get
     */
    getConfig(stackName: string, key: string, path?: boolean): Promise<ConfigValue>;
    /**
     * Returns the config map for the specified stack name, scoped to the
     * current workspace. {@link LocalWorkspace} reads this config from the
     * matching `Pulumi.stack.yaml` file.
     *
     * @param stackName
     *  The stack to read config from
     */
    getAllConfig(stackName: string): Promise<ConfigMap>;
    /**
     * Sets the specified key-value pair on the provided stack name. {@link
     * LocalWorkspace} writes this value to the matching `Pulumi.<stack>.yaml`
     * file in `Workspace.WorkDir()`.
     *
     * @param stackName
     *  The stack to operate on
     * @param key
     *  The config key to set
     * @param value
     *  The value to set
     * @param path
     *  The key contains a path to a property in a map or list to set
     */
    setConfig(stackName: string, key: string, value: ConfigValue, path?: boolean): Promise<void>;
    /**
     * Sets all values in the provided config map for the specified stack name.
     * {@link LocalWorkspace} writes the config to the matching
     * `Pulumi.<stack>.yaml` file in `Workspace.WorkDir()`.
     *
     * @param stackName
     *  The stack to operate on
     * @param config
     *  The {@link ConfigMap} to upsert against the existing config
     * @param path
     *  The keys contain a path to a property in a map or list to set
     */
    setAllConfig(stackName: string, config: ConfigMap, path?: boolean): Promise<void>;
    /**
     * Sets all config values from a JSON string for the specified stack name.
     * The JSON string should be in the format produced by "pulumi config --json".
     * Will write the config to the matching `Pulumi.<stack>.yaml` file in
     * `Workspace.workDir`.
     *
     * @param stackName
     *  The stack to operate on
     * @param configJson
     *  A JSON string containing the configuration values to set
     */
    setAllConfigJson(stackName: string, configJson: string): Promise<void>;
    /**
     * Removes the specified key-value pair on the provided stack name. Will
     * remove any matching values in the `Pulumi.<stack>.yaml` file in
     * `Workspace.WorkDir()`.
     *
     * @param stackName
     *  The stack to operate on
     * @param key
     *  The config key to remove
     * @param path
     *  The key contains a path to a property in a map or list to remove
     */
    removeConfig(stackName: string, key: string, path?: boolean): Promise<void>;
    /**
     * Removes all values in the provided key list for the specified stack name
     * Will remove any matching values in the `Pulumi.<stack>.yaml` file in
     * `Workspace.WorkDir()`.
     *
     * @param stackName
     *  The stack to operate on
     * @param keys
     *  The list of keys to remove from the underlying config
     * @param path
     *  The keys contain a path to a property in a map or list to remove
     */
    removeAllConfig(stackName: string, keys: string[], path?: boolean): Promise<void>;
    /**
     * Gets and sets the config map used with the last update for the stack
     * matching the given name. This will overwrite all configuration in the
     * `Pulumi.<stack>.yaml` file in `Workspace.WorkDir()`.
     *
     * @param stackName
     *  The stack to refresh
     */
    refreshConfig(stackName: string): Promise<ConfigMap>;
    /**
     * Returns the value associated with the specified stack name and key,
     * scoped to the {@link LocalWorkspace.}
     *
     * @param stackName
     *  The stack to read tag metadata from.
     * @param key
     *  The key to use for the tag lookup.
     */
    getTag(stackName: string, key: string): Promise<string>;
    /**
     * Sets the specified key-value pair on the stack with the given name.
     *
     * @param stackName
     *  The stack to operate on.
     * @param key
     *  The tag key to set.
     * @param value
     *  The tag value to set.
     */
    setTag(stackName: string, key: string, value: string): Promise<void>;
    /**
     * Removes the specified key-value pair on the stack with the given name.
     *
     * @param stackName
     *  The stack to operate on.
     * @param key
     *  The tag key to remove.
     */
    removeTag(stackName: string, key: string): Promise<void>;
    /**
     * Returns the tag map for the specified stack, scoped to the current
     * {@link LocalWorkspace.}
     *
     * @param stackName
     *  The stack to read tag metadata from.
     */
    listTags(stackName: string): Promise<TagMap>;
    /**
     * Returns information about the currently authenticated user.
     */
    whoAmI(): Promise<WhoAmIResult>;
    /**
     * Returns a summary of the currently selected stack, if any.
     */
    stack(): Promise<StackSummary | undefined>;
    /**
     * Returns all stacks from the underlying backend based on the provided
     * options. This queries the underlying backend and may return stacks not
     * present in the workspace as `Pulumi.<stack>.yaml` files.
     *
     * @param opts
     *  Options to customize the behavior of the list.
     */
    listStacks(opts?: ListOptions): Promise<StackSummary[]>;
    /**
     * Install plugin and language dependencies needed for the project.
     *
     * @param opts Options to customize the behavior of install.
     */
    install(opts?: InstallOptions): Promise<void>;
    /**
     * Installs a plugin in the workspace, for example to use cloud providers
     * like AWS or GCP.
     *
     * @param name
     *  The name of the plugin.
     * @param version
     *  The version of the plugin e.g. "v1.0.0".
     * @param kind
     *  The kind of plugin, defaults to "resource"
     */
    installPlugin(name: string, version: string, kind?: string): Promise<void>;
    /**
     * Installs a plugin in the workspace from a third party server.
     *
     * @param name
     *  The name of the plugin.
     * @param version
     *  The version of the plugin e.g. "v1.0.0".
     * @param server
     *  The server to install the plugin from
     */
    installPluginFromServer(name: string, version: string, server: string): Promise<void>;
    /**
     * Removes a plugin from the workspace matching the specified name and version.
     *
     * @param name
     *  The optional name of the plugin.
     * @param versionRange
     *  An optional semver range to check when removing plugins matching the
     *  given name e.g. "1.0.0", ">1.0.0".
     * @param kind
     *  The kind of plugin, defaults to "resource".
     */
    removePlugin(name?: string, versionRange?: string, kind?: string): Promise<void>;
    /**
     * Returns a list of all plugins installed in the workspace.
     */
    listPlugins(): Promise<PluginInfo[]>;
    /**
     * Exports the deployment state of the stack. This can be combined with
     * {@link importStack} to edit a stack's state (such as recovery from failed
     * deployments).
     *
     * @param stackName
     *  The name of the stack.
     */
    exportStack(stackName: string): Promise<Deployment>;
    /**
     * Imports the given deployment state into a pre-existing stack. This can be
     * combined with {@link exportStack} to edit a stack's state (such as
     * recovery from failed deployments).
     *
     * @param stackName
     *  The name of the stack.
     * @param state
     *  The stack state to import.
     */
    importStack(stackName: string, state: Deployment): Promise<void>;
    /**
     * Gets the current set of Stack outputs from the last {@link Stack.up}.
     *
     * @param stackName The name of the stack.
     */
    stackOutputs(stackName: string): Promise<OutputMap>;
    /**
     * A hook to provide additional args to every CLI commands before they are
     * executed. Provided with stack name, this function returns a list of
     * arguments to append to an invoked command (e.g. `["--config=...", ...]`)
     * Presently, {@link LocalWorkspace} does not utilize this extensibility
     * point.
     */
    serializeArgsForOp(_: string): Promise<string[]>;
    /**
     * A hook executed after every command. Called with the stack name. An
     * extensibility point to perform workspace cleanup (CLI operations may
     * create/modify a `Pulumi.stack.yaml`) {@link LocalWorkspace} does not
     * utilize this extensibility point.
     */
    postCommandCallback(_: string): Promise<void>;
    private checkRemoteSupport;
    private runPulumiCmd;
}
/**
 * Description of a stack backed by an inline (in process) Pulumi program.
 */
export interface InlineProgramArgs {
    /**
     * The associated stack name.
     */
    stackName: string;
    /**
     * The associated project name.
     */
    projectName: string;
    /**
     * The inline (in-process) Pulumi program to use with update and preview operations.
     */
    program: PulumiFn;
}
/**
 * Description of a stack backed by pre-existing local Pulumi CLI program.
 */
export interface LocalProgramArgs {
    /**
     * The associated stack name.
     */
    stackName: string;
    /**
     * The working directory of the program.
     */
    workDir: string;
}
/**
 * Extensibility options to configure a {@link LocalWorkspace;} e.g: settings to
 * seed and environment variables to pass through to every command.
 */
export interface LocalWorkspaceOptions {
    /**
     * The directory to run Pulumi commands and read settings (`Pulumi.yaml` and
     * `Pulumi.<stack>.yaml`).
     */
    workDir?: string;
    /**
     * The directory to override for CLI metadata
     */
    pulumiHome?: string;
    /**
     * The underlying Pulumi CLI.
     */
    pulumiCommand?: PulumiCommand;
    /**
     * The inline program {@link PulumiFn} to be used for preview/update
     * operations, if any. If none is specified, the stack will refer to
     * {@link ProjectSettings} for this information.
     */
    program?: PulumiFn;
    /**
     * The image to use for the remote Pulumi operation.
     */
    remoteExecutorImage?: ExecutorImage;
    /**
     * Environment values scoped to the current workspace. These will be supplied to every Pulumi command.
     */
    envVars?: {
        [key: string]: string;
    };
    /**
     * The secrets provider to use for encryption and decryption of stack secrets.
     * See: https://www.pulumi.com/docs/intro/concepts/secrets/#available-encryption-providers
     */
    secretsProvider?: string;
    /**
     * The settings object for the current project.
     */
    projectSettings?: ProjectSettings;
    /**
     * A map of stack names and corresponding settings objects.
     */
    stackSettings?: {
        [key: string]: StackSettings;
    };
}
export interface InstallOptions {
    /**
     * Skip installing plugins
     */
    noPlugins?: boolean;
    /**
     * Skip installing dependencies
     */
    noDependencies?: boolean;
    /**
     * Reinstall plugins even if they already exist
     */
    reinstall?: boolean;
    /**
     * Use language version tools to setup the language runtime before installing the dependencies.
     * For Python this will use `pyenv` to install the Python version specified in a
     * `.python-version` file. For Nodejs this will use `fnm` to install the Node.js version
     * specified in a `.nvmrc` or `.node-version file.
     */
    useLanguageVersionTools?: boolean;
}
export interface ListOptions {
    /**
     * List all stacks instead of just stacks for the current project
     */
    all?: boolean;
}
export interface RemoveOptions {
    /**
     * Forces deletion of the stack, leaving behind any resources managed by the stack
     */
    force?: boolean;
    /**
     * Do not delete the corresponding Pulumi.<stack-name>.yaml configuration file for the stack
     */
    preserveConfig?: boolean;
}
