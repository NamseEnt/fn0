"use strict";
// Copyright 2016-2021, Pulumi Corporation.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const grpc = __importStar(require("@grpc/grpc-js"));
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const callbacks_1 = require("./callbacks");
const debuggable_1 = require("./debuggable");
const state_1 = require("./state");
const engrpc = __importStar(require("../proto/engine_grpc_pb"));
const engproto = __importStar(require("../proto/engine_pb"));
const resrpc = __importStar(require("../proto/resource_grpc_pb"));
const resproto = __importStar(require("../proto/resource_pb"));
const emptyproto = __importStar(require("google-protobuf/google/protobuf/empty_pb"));
/*
  Raises the gRPC Max Message size from `4194304` (4mb) to `419430400` (400mb).
 */
const maxRPCMessageSize = 1024 * 1024 * 400;
/*
  This is the time a message can be received by the GRPC server and wait in the queue without being handled. If there is
  blocking happening in the pulumi program, and/or there are a lot of requests and other tasks to process by the event
  loop, this can take longer than the default 30 seconds. Requests that take longer end up being cancelled, causing the
  operation to fail.
*/
const serverMaxUnrequestedTimeInServer = 30 * 60; // half an hour in seconds
/**
 * @internal
 */
exports.grpcChannelOptions = {
    "grpc.max_receive_message_length": maxRPCMessageSize,
    "grpc.server_max_unrequested_time_in_server": serverMaxUnrequestedTimeInServer,
};
/**
 * excessiveDebugOutput enables, well, pretty excessive debug output pertaining
 * to resources and properties.
 */
exports.excessiveDebugOutput = false;
let monitor;
let engine;
/**
 * Resets NodeJS runtime global state (such as RPC clients), and sets NodeJS
 * runtime option environment variables to the specified values.
 */
function resetOptions(project, stack, parallel, engineAddr, monitorAddr, preview, organization) {
    const store = state_1.getStore();
    monitor = undefined;
    engine = undefined;
    store.settings.monitor = undefined;
    store.settings.engine = undefined;
    store.settings.rpcDone = Promise.resolve();
    store.settings.featureSupport = {};
    // reset node specific environment variables in the process
    store.settings.options.project = project;
    store.settings.options.stack = stack;
    store.settings.options.dryRun = preview;
    store.settings.options.parallel = parallel;
    store.settings.options.monitorAddr = monitorAddr;
    store.settings.options.engineAddr = engineAddr;
    store.settings.options.organization = organization;
    store.leakCandidates = new Set();
    store.logErrorCount = 0;
    store.stackResource = undefined;
    store.supportsSecrets = false;
    store.supportsResourceReferences = false;
    store.supportsOutputValues = false;
    store.supportsDeletedWith = false;
    store.supportsReplaceWith = false;
    store.supportsAliasSpecs = false;
    store.supportsTransforms = false;
    store.supportsInvokeTransforms = false;
    store.supportsParameterization = false;
    store.callbacks = undefined;
}
exports.resetOptions = resetOptions;
function setMockOptions(mockMonitor, project, stack, preview, organization) {
    const opts = options();
    resetOptions(project || opts.project || "project", stack || opts.stack || "stack", opts.parallel || -1, opts.engineAddr || "", "mock", preview || false, organization || "");
    const { settings } = state_1.getStore();
    settings.monitor = mockMonitor;
    monitor = mockMonitor;
}
exports.setMockOptions = setMockOptions;
/**
 * @internal
 *  Used only for testing purposes.
 */
function _setIsDryRun(val) {
    const { settings } = state_1.getStore();
    settings.options.dryRun = val;
}
exports._setIsDryRun = _setIsDryRun;
/**
 * Returns true if we are currently doing a preview.
 *
 * When writing unit tests, you can set this flag via either `setMocks` or
 * `_setIsDryRun`.
 */
function isDryRun() {
    return options().dryRun === true;
}
exports.isDryRun = isDryRun;
/**
 * Returns a promise that when resolved tells you if the resource monitor we are
 * connected to is able to support a particular feature.
 *
 * @internal
 */
async function monitorSupportsFeature(monitorClient, feature) {
    const req = new resproto.SupportsFeatureRequest();
    req.setId(feature);
    const result = await new Promise((resolve, reject) => {
        monitorClient.supportsFeature(req, (err, resp) => {
            // Back-compat case - if the monitor doesn't let us ask if it supports a feature, it doesn't support
            // any features.
            if (err && err.code === grpc.status.UNIMPLEMENTED) {
                return resolve(false);
            }
            if (err) {
                return reject(err);
            }
            if (resp === undefined) {
                return reject(new Error("No response from resource monitor"));
            }
            return resolve(resp.getHassupport());
        });
    });
    return result;
}
/**
 * Queries the resource monitor for its capabilities and sets the appropriate
 * flags in the store.
 *
 * @internal
 **/
async function awaitFeatureSupport() {
    const monitorRef = getMonitor();
    if (monitorRef !== undefined) {
        const store = state_1.getStore();
        const [secrets, resourceReferences, outputValues, deletedWith, replaceWith, aliasSpecs, transforms, invokeTransforms, parameterization, resourceHooks,] = await Promise.all([
            "secrets",
            "resourceReferences",
            "outputValues",
            "deletedWith",
            "replaceWith",
            "aliasSpecs",
            "transforms",
            "invokeTransforms",
            "parameterization",
            "resourceHooks",
        ].map((feature) => monitorSupportsFeature(monitorRef, feature)));
        store.supportsSecrets = secrets;
        store.supportsResourceReferences = resourceReferences;
        store.supportsOutputValues = outputValues;
        store.supportsDeletedWith = deletedWith;
        store.supportsReplaceWith = replaceWith;
        store.supportsAliasSpecs = aliasSpecs;
        store.supportsTransforms = transforms;
        store.supportsInvokeTransforms = invokeTransforms;
        store.supportsParameterization = parameterization;
        store.supportsResourceHooks = resourceHooks;
    }
}
exports.awaitFeatureSupport = awaitFeatureSupport;
/**
 * @internal
 *  Used only for testing purposes.
 */
function _reset() {
    resetOptions("", "", -1, "", "", false, "");
}
exports._reset = _reset;
/**
 * Returns true if we will resolve missing outputs to inputs during preview
 * (`PULUMI_ENABLE_LEGACY_APPLY`).
 */
function isLegacyApplyEnabled() {
    return options().legacyApply === true;
}
exports.isLegacyApplyEnabled = isLegacyApplyEnabled;
/**
 * Returns true if we will cache serialized dynamic providers on the program
 * side (the default is true).
 */
function cacheDynamicProviders() {
    return options().cacheDynamicProviders === true;
}
exports.cacheDynamicProviders = cacheDynamicProviders;
/**
 * Get the organization being run by the current update.
 */
function getOrganization() {
    const organization = options().organization;
    if (organization) {
        return organization;
    }
    // If the organization is missing, specialize the error.
    // Throw an error if test mode is enabled, instructing how to manually configure the organization:
    throw new Error("Missing organization name; for test mode, please call `pulumi.runtime.setMocks`");
}
exports.getOrganization = getOrganization;
/**
 * @internal
 *  Used only for testing purposes.
 */
function _setOrganization(val) {
    const { settings } = state_1.getStore();
    settings.options.organization = val;
    return settings.options.organization;
}
exports._setOrganization = _setOrganization;
/**
 * Get the project being run by the current update.
 */
function getProject() {
    const { project } = options();
    return project || "";
}
exports.getProject = getProject;
/**
 * Get the project root directory.  This is the location of the Pulumi.yaml file.
 */
function getRootDirectory() {
    const { rootDirectory: rootDirectory } = options();
    return rootDirectory || "";
}
exports.getRootDirectory = getRootDirectory;
/**
 * @internal
 *  Used only for testing purposes.
 */
function _setProject(val) {
    const { settings } = state_1.getStore();
    settings.options.project = val;
    return settings.options.project;
}
exports._setProject = _setProject;
/**
 * Get the stack being targeted by the current update.
 */
function getStack() {
    const { stack } = options();
    return stack || "";
}
exports.getStack = getStack;
/**
 * @internal
 *  Used only for testing purposes.
 */
function _setStack(val) {
    const { settings } = state_1.getStore();
    settings.options.stack = val;
    return settings.options.stack;
}
exports._setStack = _setStack;
/**
 * Returns true if we are currently connected to a resource monitoring service.
 */
function hasMonitor() {
    const { settings } = state_1.getStore();
    return (!!monitor && !!options().monitorAddr) || !!settings.monitor;
}
exports.hasMonitor = hasMonitor;
/**
 * Returns the current resource monitoring service client for RPC
 * communications.
 */
function getMonitor() {
    const { settings } = state_1.getStore();
    const addr = options().monitorAddr;
    if (state_1.getLocalStore() === undefined && addr !== "mock") {
        if (monitor === undefined) {
            if (addr) {
                // Lazily initialize the RPC connection to the monitor.
                monitor = new resrpc.ResourceMonitorClient(addr, grpc.credentials.createInsecure(), exports.grpcChannelOptions);
                settings.options.monitorAddr = addr;
            }
        }
        return monitor;
    }
    else {
        if (settings.monitor === undefined) {
            if (addr) {
                // Lazily initialize the RPC connection to the monitor.
                settings.monitor = new resrpc.ResourceMonitorClient(addr, grpc.credentials.createInsecure(), exports.grpcChannelOptions);
                settings.options.monitorAddr = addr;
            }
        }
        return settings.monitor;
    }
}
exports.getMonitor = getMonitor;
/**
 * Waits for any pending stack transforms to register.
 */
async function awaitStackRegistrations() {
    const store = state_1.getStore();
    const callbacks = store.callbacks;
    if (callbacks === undefined) {
        return;
    }
    return await callbacks.awaitStackRegistrations();
}
exports.awaitStackRegistrations = awaitStackRegistrations;
/**
 * Returns the current callbacks for RPC communications.
 */
function getCallbacks() {
    const store = state_1.getStore();
    const callbacks = store.callbacks;
    if (callbacks !== undefined) {
        return callbacks;
    }
    const monitorRef = getMonitor();
    if (monitorRef === undefined) {
        return undefined;
    }
    const callbackServer = new callbacks_1.CallbackServer(monitorRef);
    store.callbacks = callbackServer;
    return callbackServer;
}
exports.getCallbacks = getCallbacks;
let syncInvokes;
/**
 * @internal
 */
function tryGetSyncInvokes() {
    const syncDir = options().syncDir;
    if (syncInvokes === undefined && syncDir) {
        const requests = fs.openSync(path.join(syncDir, "invoke_req"), fs.constants.O_WRONLY | fs.constants.O_SYNC);
        const responses = fs.openSync(path.join(syncDir, "invoke_res"), fs.constants.O_RDONLY | fs.constants.O_SYNC);
        syncInvokes = { requests, responses };
    }
    return syncInvokes;
}
exports.tryGetSyncInvokes = tryGetSyncInvokes;
/**
 * Returns true if we are currently connected to an engine.
 */
function hasEngine() {
    return !!engine && !!options().engineAddr;
}
exports.hasEngine = hasEngine;
/**
 * Returns the current engine, if any, for RPC communications back to the
 * resource engine.
 */
function getEngine() {
    const { settings } = state_1.getStore();
    if (state_1.getLocalStore() === undefined) {
        if (engine === undefined) {
            const addr = options().engineAddr;
            if (addr) {
                // Lazily initialize the RPC connection to the engine.
                engine = new engrpc.EngineClient(addr, grpc.credentials.createInsecure(), exports.grpcChannelOptions);
            }
        }
        return engine;
    }
    else {
        if (settings.engine === undefined) {
            const addr = options().engineAddr;
            if (addr) {
                // Lazily initialize the RPC connection to the engine.
                settings.engine = new engrpc.EngineClient(addr, grpc.credentials.createInsecure(), exports.grpcChannelOptions);
            }
        }
        return settings.engine;
    }
}
exports.getEngine = getEngine;
function terminateRpcs() {
    const store = state_1.getStore();
    store.terminated = true;
    disconnectSync();
}
exports.terminateRpcs = terminateRpcs;
/**
 * Returns true if resource operations should be serialized.
 */
function serialize() {
    return options().parallel === 1;
}
exports.serialize = serialize;
/**
 * Returns the options from the environment, which is the source of truth.
 * Options are global per process.
 *
 * For CLI driven programs, `pulumi-language-nodejs` sets environment variables
 * prior to the user program loading, meaning that options could be loaded up
 * front and cached. Automation API and multi-language components introduced
 * more complex lifecycles for runtime `options()`. These language hosts manage
 * the lifecycle of options manually throughout the lifetime of the NodeJS
 * process. In addition, NodeJS module resolution can lead to duplicate copies
 * of `@pulumi/pulumi` and thus duplicate options objects that may not be synced
 * if options are cached upfront. Mutating options must write to the environment
 * and reading options must always read directly from the environment.
 */
function options() {
    const { settings } = state_1.getStore();
    return settings.options;
}
/**
 * Permanently disconnects from the server, closing the connections. It waits
 * for the existing RPC queue to drain.  If any RPCs come in afterwards,
 * however, they will crash the process.
 *
 * If `signalShutdown` is true, signal to the monitor that we're ready to
 * shutdown. This will wait until the monitor has completed all steps,
 * including deletion steps.
 */
async function disconnect(signalShutdown = false) {
    await waitForRPCs();
    if (signalShutdown) {
        await signalAndWaitForShutdown();
    }
    disconnectSync();
}
exports.disconnect = disconnect;
/**
 * Waits for the existing RPC queue to drain.
 *
 * @internal
 */
function waitForRPCs() {
    const localStore = state_1.getStore();
    let done;
    const closeCallback = async () => {
        if (done !== localStore.settings.rpcDone) {
            // If the done promise has changed, some activity occurred in between callbacks.  Wait again.
            done = localStore.settings.rpcDone;
            return debuggable_1.debuggablePromise(done.then(closeCallback), "disconnect");
        }
        return Promise.resolve();
    };
    return closeCallback();
}
exports.waitForRPCs = waitForRPCs;
/**
 * Signals to the monitor that we're ready to shutdown. This will wait until the
 * monitor has completed all steps, including deletion steps.
 *
 * @internal
 */
function signalAndWaitForShutdown() {
    return new Promise((resolve, reject) => {
        const monitorRef = getMonitor();
        if (!monitorRef) {
            return resolve();
        }
        monitorRef.signalAndWaitForShutdown(new emptyproto.Empty(), (err, _) => {
            if (err) {
                // If we are running against an older version of the CLI,
                // SignalAndWaitForShutdown might not be implemented. This is
                // mostly fine, but means that delete hooks do not work. Since
                // we check if the CLI supports the `resourceHook` feature when
                // registering hooks, it's fine to ignore the `UNIMPLEMENTED`
                // error here.
                // If we get `UNAVAILABLE`, the monitor was already shutdown, likely
                // due to an error, and we can ignore the GRPC error here, since
                // Pulumi will have notified the user.
                if (err && (err.code === grpc.status.UNIMPLEMENTED || err.code === grpc.status.UNAVAILABLE)) {
                    return resolve();
                }
                return reject(new Error(`Error while signaling shutdown: ${err}`));
            }
            return resolve();
        });
    });
}
exports.signalAndWaitForShutdown = signalAndWaitForShutdown;
/**
 * Returns the configured number of process listeners available.
 */
function getMaximumListeners() {
    const { settings } = state_1.getStore();
    return settings.options.maximumProcessListeners;
}
exports.getMaximumListeners = getMaximumListeners;
/**
 * Permanently disconnects from the server, closing the connections. Unlike
 * `disconnect`. it does not wait for the existing RPC queue to drain. Any RPCs
 * that come in after this call will crash the process.
 */
function disconnectSync() {
    // Otherwise, actually perform the close activities (ignoring errors and crashes).
    const store = state_1.getStore();
    if (store.callbacks) {
        store.callbacks.shutdown();
        store.callbacks = undefined;
    }
    if (monitor) {
        try {
            monitor.close();
        }
        catch (err) {
            // ignore.
        }
        monitor = undefined;
    }
    if (engine) {
        try {
            engine.close();
        }
        catch (err) {
            // ignore.
        }
        engine = undefined;
    }
}
exports.disconnectSync = disconnectSync;
/**
 * Registers a pending call to ensure that we don't prematurely disconnect from
 * the server.  It returns a function that, when invoked, signals that the RPC
 * has completed.
 */
function rpcKeepAlive() {
    const localStore = state_1.getStore();
    let done = undefined;
    const donePromise = debuggable_1.debuggablePromise(new Promise((resolve) => {
        done = resolve;
        return done;
    }), "rpcKeepAlive");
    localStore.settings.rpcDone = localStore.settings.rpcDone.then(() => donePromise);
    return done;
}
exports.rpcKeepAlive = rpcKeepAlive;
/**
 * Returns if the engine supports package references and parameterized providers.
 */
function supportsParameterization() {
    return state_1.getStore().supportsParameterization;
}
exports.supportsParameterization = supportsParameterization;
/**
 * Registers a resource that will become the default parent for all resources
 * without explicit parents.
 */
async function setRootResource(res) {
    // This is the first async point of program startup where we can query the resource monitor for its capabilities.
    await awaitFeatureSupport();
    const engineRef = getEngine();
    if (!engineRef) {
        return Promise.resolve();
    }
    // Back-compat case - Try to set the root URN for SxS old SDKs that expect the engine to roundtrip the
    // stack URN.
    const req = new engproto.SetRootResourceRequest();
    const urn = await res.urn.promise();
    req.setUrn(urn);
    return new Promise((resolve, reject) => {
        engineRef.setRootResource(req, (err, resp) => {
            // Back-compat case - if the engine we're speaking to isn't aware that it can save and load root
            // resources, just ignore there's nothing we can do.
            if (err && err.code === grpc.status.UNIMPLEMENTED) {
                return resolve();
            }
            if (err) {
                return reject(err);
            }
            return resolve();
        });
    });
}
exports.setRootResource = setRootResource;
//# sourceMappingURL=settings.js.map