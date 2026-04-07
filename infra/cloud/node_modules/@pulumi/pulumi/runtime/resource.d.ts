import { Inputs, Output } from "../output";
import { Alias, Resource, ResourceOptions, ResourceHook, URN } from "../resource";
import * as aliasproto from "../proto/alias_pb";
export interface SourcePosition {
    uri: string;
    line: number;
    column: number;
}
/**
 * Get an existing resource's state from the engine.
 */
export declare function getResource(res: Resource, parent: Resource | undefined, props: Inputs, custom: boolean, urn: string): void;
/**
 * Reads an existing custom resource's state from the resource monitor.  Note
 * that resources read in this way will not be part of the resulting stack's
 * state, as they are presumed to belong to another.
 */
export declare function readResource(res: Resource, parent: Resource | undefined, t: string, name: string, props: Inputs, opts: ResourceOptions, sourcePosition?: SourcePosition, stackTrace?: (SourcePosition | undefined)[], packageRef?: Promise<string | undefined>): void;
export declare function mapAliasesForRequest(aliases: (URN | Alias)[] | undefined, parentURN?: URN): Promise<aliasproto.Alias[]>;
/**
 * registerResource registers a new resource object with a given type `t` and
 * `name`. It returns the auto-generated URN and the ID that will resolve after
 * the deployment has completed.  All properties will be initialized to property
 * objects that the registration operation will resolve at the right time (or
 * remain unresolved for deployments).
 */
export declare function registerResource(res: Resource, parent: Resource | undefined, t: string, name: string, custom: boolean, remote: boolean, newDependency: (urn: URN) => Resource, props: Inputs, opts: ResourceOptions, sourcePosition?: SourcePosition, stackTrace?: (SourcePosition | undefined)[], packageRef?: Promise<string | undefined>): void;
/**
 * Completes a resource registration, attaching an optional set of computed
 * outputs.
 */
export declare function registerResourceOutputs(res: Resource, outputs: Inputs | Promise<Inputs> | Output<Inputs>): void;
export declare function registerResourceHook(hook: ResourceHook): Promise<void>;
