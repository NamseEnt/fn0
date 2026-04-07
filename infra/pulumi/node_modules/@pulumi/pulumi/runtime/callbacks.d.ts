import * as callproto from "../proto/callback_pb";
import * as resrpc from "../proto/resource_grpc_pb";
import { ResourceHook, ResourceTransform } from "../resource";
import { InvokeTransform } from "../invoke";
export interface ICallbackServer {
    registerTransform(callback: ResourceTransform): Promise<callproto.Callback>;
    registerStackTransform(callback: ResourceTransform): void;
    registerStackInvokeTransform(callback: InvokeTransform): void;
    registerStackInvokeTransformAsync(callback: InvokeTransform): Promise<callproto.Callback>;
    registerResourceHook(hook: ResourceHook): Promise<void>;
    shutdown(): void;
    awaitStackRegistrations(): Promise<void>;
}
export declare class CallbackServer implements ICallbackServer {
    private readonly _callbacks;
    private readonly _monitor;
    private readonly _server;
    private readonly _target;
    private _pendingRegistrations;
    private _awaitQueue;
    constructor(monitor: resrpc.IResourceMonitorClient);
    awaitStackRegistrations(): Promise<void>;
    shutdown(): void;
    private invoke;
    registerTransform(transform: ResourceTransform): Promise<callproto.Callback>;
    registerStackTransform(transform: ResourceTransform): void;
    registerStackInvokeTransformAsync(transform: InvokeTransform): Promise<callproto.Callback>;
    registerStackInvokeTransform(transform: InvokeTransform): void;
    registerResourceHook(hook: ResourceHook): Promise<void>;
}
