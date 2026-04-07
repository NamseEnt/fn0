import { ComponentResource } from "../resource";
import { ICallbackServer } from "./callbacks";
import * as engrpc from "../proto/engine_grpc_pb";
import * as resrpc from "../proto/resource_grpc_pb";
/**
 * excessiveDebugOutput enables, well, pretty excessive debug output pertaining
 * to resources and properties.
 */
export declare const excessiveDebugOutput: boolean;
/**
 * {@link Options} is a bag of settings that controls the behavior of previews
 * and deployments.
 */
export interface Options {
    /**
     * The name of the current project.
     */
    readonly project?: string;
    /**
     * The root directory of the current project. This is the location of the Pulumi.yaml file.
     */
    readonly rootDirectory?: string;
    /**
     * The name of the current stack being deployed into.
     */
    readonly stack?: string;
    /**
     * The degree of parallelism for resource operations (default is serial).
     */
    readonly parallel?: number;
    /**
     * A connection string to the engine's RPC, in case we need to reestablish.
     */
    readonly engineAddr?: string;
    /**
     * A connection string to the monitor's RPC, in case we need to reestablish.
     */
    readonly monitorAddr?: string;
    /**
     * Whether we are performing a preview (true) or a real deployment (false).
     */
    readonly dryRun?: boolean;
    /**
     * True if we're in testing mode (allows execution without the CLI).
     */
    readonly testModeEnabled?: boolean;
    /**
     * True if we will resolve missing outputs to inputs during preview.
     */
    readonly legacyApply?: boolean;
    /**
     * True if we will cache serialized dynamic providers on the program side.
     */
    readonly cacheDynamicProviders?: boolean;
    /**
     * The name of the current organization.
     */
    readonly organization?: string;
    /**
     * A directory containing the send/receive files for making synchronous
     * invokes to the engine.
     */
    readonly syncDir?: string;
}
/**
 * Resets NodeJS runtime global state (such as RPC clients), and sets NodeJS
 * runtime option environment variables to the specified values.
 */
export declare function resetOptions(project: string, stack: string, parallel: number, engineAddr: string, monitorAddr: string, preview: boolean, organization: string): void;
export declare function setMockOptions(mockMonitor: any, project?: string, stack?: string, preview?: boolean, organization?: string): void;
/**
 * Returns true if we are currently doing a preview.
 *
 * When writing unit tests, you can set this flag via either `setMocks` or
 * `_setIsDryRun`.
 */
export declare function isDryRun(): boolean;
/**
 * Returns true if we will resolve missing outputs to inputs during preview
 * (`PULUMI_ENABLE_LEGACY_APPLY`).
 */
export declare function isLegacyApplyEnabled(): boolean;
/**
 * Returns true if we will cache serialized dynamic providers on the program
 * side (the default is true).
 */
export declare function cacheDynamicProviders(): boolean;
/**
 * Get the organization being run by the current update.
 */
export declare function getOrganization(): string;
/**
 * Get the project being run by the current update.
 */
export declare function getProject(): string;
/**
 * Get the project root directory.  This is the location of the Pulumi.yaml file.
 */
export declare function getRootDirectory(): string;
/**
 * Get the stack being targeted by the current update.
 */
export declare function getStack(): string;
/**
 * Returns true if we are currently connected to a resource monitoring service.
 */
export declare function hasMonitor(): boolean;
/**
 * Returns the current resource monitoring service client for RPC
 * communications.
 */
export declare function getMonitor(): resrpc.IResourceMonitorClient | undefined;
/**
 * Waits for any pending stack transforms to register.
 */
export declare function awaitStackRegistrations(): Promise<void>;
/**
 * Returns the current callbacks for RPC communications.
 */
export declare function getCallbacks(): ICallbackServer | undefined;
/**
 * Returns true if we are currently connected to an engine.
 */
export declare function hasEngine(): boolean;
/**
 * Returns the current engine, if any, for RPC communications back to the
 * resource engine.
 */
export declare function getEngine(): engrpc.IEngineClient | undefined;
export declare function terminateRpcs(): void;
/**
 * Returns true if resource operations should be serialized.
 */
export declare function serialize(): boolean;
/**
 * Permanently disconnects from the server, closing the connections. It waits
 * for the existing RPC queue to drain.  If any RPCs come in afterwards,
 * however, they will crash the process.
 *
 * If `signalShutdown` is true, signal to the monitor that we're ready to
 * shutdown. This will wait until the monitor has completed all steps,
 * including deletion steps.
 */
export declare function disconnect(signalShutdown?: boolean): Promise<void>;
/**
 * Returns the configured number of process listeners available.
 */
export declare function getMaximumListeners(): number;
/**
 * Permanently disconnects from the server, closing the connections. Unlike
 * `disconnect`. it does not wait for the existing RPC queue to drain. Any RPCs
 * that come in after this call will crash the process.
 */
export declare function disconnectSync(): void;
/**
 * Registers a pending call to ensure that we don't prematurely disconnect from
 * the server.  It returns a function that, when invoked, signals that the RPC
 * has completed.
 */
export declare function rpcKeepAlive(): () => void;
/**
 * Returns if the engine supports package references and parameterized providers.
 */
export declare function supportsParameterization(): boolean;
/**
 * Registers a resource that will become the default parent for all resources
 * without explicit parents.
 */
export declare function setRootResource(res: ComponentResource): Promise<void>;
