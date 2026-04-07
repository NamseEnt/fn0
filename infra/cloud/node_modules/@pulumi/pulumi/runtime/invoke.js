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
const log = __importStar(require("../log"));
const output_1 = require("../output");
const debuggable_1 = require("./debuggable");
const rpc_1 = require("./rpc");
const settings_1 = require("./settings");
const resource_1 = require("../resource");
const utils = __importStar(require("../utils"));
const dependsOn_1 = require("./dependsOn");
const gstruct = __importStar(require("google-protobuf/google/protobuf/struct_pb"));
const resourceproto = __importStar(require("../proto/resource_pb"));
/**
 * Dynamically invokes the function `tok`, which is offered by a provider
 * plugin. `invoke` behaves differently in the case that options contains
 * `{async:true}` or not.
 *
 * In the case where `{async:true}` is present in the options bag:
 *
 * 1. the result of `invoke` will be a Promise resolved to the result value of
 *    the provider plugin.
 *
 * 2. the `props` inputs can be a bag of computed values (including, `T`s,
 *   `Promise<T>`s, `Output<T>`s etc.).
 *
 * In the case where `{ async:true }` is not present in the options bag:
 *
 * 1. the result of `invoke` will be a Promise resolved to the result value of
 *    the provider call. However, that Promise will *also* have the respective
 *    values of the Provider result exposed directly on it as properties.
 *
 * 2. The inputs must be a bag of simple values, and the result is the result
 *    that the Provider produced.
 *
 * Simple values are:
 *
 *  1. `undefined`, `null`, string, number or boolean values.
 *  2. arrays of simple values.
 *  3. objects containing only simple values.
 *
 * Importantly, simple values do *not* include:
 *
 *  1. `Promise`s
 *  2. `Output`s
 *  3. `Asset`s or `Archive`s
 *  4. `Resource`s.
 *
 * All of these contain async values that would prevent `invoke from being able
 * to operate synchronously.
 */
function invoke(tok, props, opts = {}, packageRef) {
    const optsCopy = { ...opts };
    if ("dependsOn" in optsCopy) {
        // DependsOn is only allowed for invokeOutput.
        //@ts-ignore
        optsCopy["dependsOn"] = undefined;
    }
    return invokeAsync(tok, props, optsCopy, packageRef).then((response) => {
        // ignore secrets for plain invoke
        const { result } = response;
        return result;
    });
}
exports.invoke = invoke;
/**
 * Similar to the plain `invoke` but returns the response as an output, maintaining
 * secrets of the response, if any.
 */
function invokeOutput(tok, props, opts = {}, packageRef) {
    const [output, resolve] = createOutput(`invoke(${tok})`);
    invokeAsync(tok, props, opts, packageRef, true /* checkDependencies */)
        .then((response) => {
        const { result, isKnown, containsSecrets, dependencies } = response;
        resolve(result, isKnown, containsSecrets, dependencies, undefined);
    })
        .catch((err) => {
        resolve(undefined, true, false, [], err);
    });
    return output;
}
exports.invokeOutput = invokeOutput;
function extractSingleValue(result) {
    if (result === undefined) {
        return result;
    }
    // assume outputs has at least one key
    const keys = Object.keys(result);
    // return the first key's value from the outputs
    return result[keys[0]];
}
/*
 * Dynamically invokes the function `tok`, which is offered by a
 * provider plugin. Similar to `invoke`, but returns a single value instead of
 * an object with a single key.
 */
function invokeSingle(tok, props, opts = {}, packageRef) {
    return invokeAsync(tok, props, opts, packageRef).then((response) => {
        // ignore secrets for plain invoke
        const { result } = response;
        return extractSingleValue(result);
    });
}
exports.invokeSingle = invokeSingle;
/**
 * Similar to the plain `invokeSingle` but returns the response as an output, maintaining
 * secrets of the response, if any.
 */
function invokeSingleOutput(tok, props, opts = {}, packageRef) {
    const [output, resolve] = createOutput(`invokeSingleOutput(${tok})`);
    invokeAsync(tok, props, opts, packageRef, true /* checkDependencies */)
        .then((response) => {
        const { result, isKnown, containsSecrets, dependencies } = response;
        const value = extractSingleValue(result);
        resolve(value, isKnown, containsSecrets, dependencies, undefined);
    })
        .catch((err) => {
        resolve(undefined, true, false, [], err);
    });
    return output;
}
exports.invokeSingleOutput = invokeSingleOutput;
async function invokeAsync(tok, props, opts, packageRef, checkDependencies) {
    const label = `Invoking function: tok=${tok} asynchronously`;
    log.debug(label + (settings_1.excessiveDebugOutput ? `, props=${JSON.stringify(props)}` : ``));
    await settings_1.awaitStackRegistrations();
    // Wait for all values to be available, and then perform the RPC.
    const done = settings_1.rpcKeepAlive();
    try {
        // The direct dependencies of the invoke call from the dependsOn option.
        const dependsOnDeps = await dependsOn_1.gatherExplicitDependencies(opts.dependsOn);
        // The dependencies of the inputs to the invoke call.
        const [serialized, deps] = await rpc_1.serializePropertiesReturnDeps(`invoke:${tok}`, props);
        if (rpc_1.containsUnknownValues(serialized)) {
            // if any of the input properties are unknown,
            // make sure the entire response is marked as unknown
            return {
                result: {},
                isKnown: false,
                containsSecrets: false,
                dependencies: [],
            };
        }
        // Only check the resource dependencies for output form invokes. For
        // plain invokes, we do not want to check the dependencies. Technically,
        // these should only receive plain arguments, but this is not strictly
        // enforced, and in practice people pass in outputs. This happens to
        // work because we serialize the arguments.
        if (checkDependencies) {
            // If we depend on any CustomResources, we need to ensure that their
            // ID is known before proceeding. If it is not known, we will return
            // an unknown result.
            const resourcesToWaitFor = new Set(dependsOnDeps);
            // Add the dependencies from the inputs to the set of resources to wait for.
            for (const resourceDeps of deps.values()) {
                for (const value of resourceDeps.values()) {
                    resourcesToWaitFor.add(value);
                }
            }
            // The expanded set of dependencies, including children of components.
            const expandedDeps = await dependsOn_1.getAllTransitivelyReferencedResources(resourcesToWaitFor, new Set());
            // Ensure that all resource IDs are known before proceeding.
            for (const dep of expandedDeps.values()) {
                // DependencyResources inherit from CustomResource, but they don't set the id. Skip them.
                if (resource_1.CustomResource.isInstance(dep) && dep.id) {
                    const known = await dep.id.isKnown;
                    if (!known) {
                        return {
                            result: {},
                            isKnown: false,
                            containsSecrets: false,
                            dependencies: [],
                        };
                    }
                }
            }
        }
        log.debug(`Invoke RPC prepared: tok=${tok}` + settings_1.excessiveDebugOutput ? `, obj=${JSON.stringify(serialized)}` : ``);
        // Fetch the monitor and make an RPC request.
        const monitor = settings_1.getMonitor();
        const provider = await resource_1.ProviderResource.register(getProvider(tok, opts));
        // keep track of the the secretness of the inputs
        // if any of the inputs are secret, the invoke response should be marked as secret
        const [plainInputs, inputsContainSecrets] = rpc_1.unwrapSecretValues(serialized);
        const req = await createInvokeRequest(tok, plainInputs, provider, opts, packageRef);
        const resp = await debuggable_1.debuggablePromise(new Promise((innerResolve, innerReject) => monitor.invoke(req, (err, innerResponse) => {
            log.debug(`Invoke RPC finished: tok=${tok}; err: ${err}, resp: ${innerResponse}`);
            if (err) {
                // If the monitor is unavailable, it is in the process of shutting down or has already
                // shut down. Don't emit an error and don't do any more RPCs, just exit.
                if (err.code === grpc.status.UNAVAILABLE || err.code === grpc.status.CANCELLED) {
                    settings_1.terminateRpcs();
                    err.message = "Resource monitor is terminating";
                    innerReject(err);
                    return;
                }
                // If the RPC failed, rethrow the error with a native exception and the message that
                // the engine provided - it's suitable for user presentation.
                innerReject(new Error(err.details));
            }
            else {
                innerResolve(innerResponse);
            }
        })), label);
        const flatDependencies = dependsOnDeps;
        for (const dep of deps.values()) {
            for (const d of dep) {
                if (!flatDependencies.includes(d)) {
                    flatDependencies.push(d);
                }
            }
        }
        // Finally propagate any other properties that were given to us as outputs.
        const deserialized = deserializeResponse(tok, resp);
        return {
            result: deserialized.result,
            containsSecrets: deserialized.containsSecrets || inputsContainSecrets,
            dependencies: flatDependencies,
            isKnown: true,
        };
    }
    finally {
        done();
    }
}
async function createInvokeRequest(tok, serialized, provider, opts, packageRef) {
    if (provider !== undefined && typeof provider !== "string") {
        throw new Error("Incorrect provider type.");
    }
    const obj = gstruct.Struct.fromJavaScript(serialized);
    let packageRefStr = undefined;
    if (packageRef !== undefined) {
        packageRefStr = await packageRef;
        if (packageRefStr !== undefined) {
            // If we have a package reference we can clear some of the resource options
            opts.version = undefined;
            opts.pluginDownloadURL = undefined;
        }
    }
    const req = new resourceproto.ResourceInvokeRequest();
    req.setTok(tok);
    req.setArgs(obj);
    req.setProvider(provider || "");
    req.setVersion(opts.version || "");
    req.setPlugindownloadurl(opts.pluginDownloadURL || "");
    req.setAcceptresources(!utils.disableResourceReferences);
    req.setPackageref(packageRefStr || "");
    return req;
}
function getProvider(tok, opts) {
    return opts.provider ? opts.provider : opts.parent ? opts.parent.getProvider(tok) : undefined;
}
function deserializeResponse(tok, resp) {
    const failures = resp.getFailuresList();
    if (failures?.length) {
        let reasons = "";
        for (let i = 0; i < failures.length; i++) {
            if (reasons !== "") {
                reasons += "; ";
            }
            reasons += `${failures[i].getReason()} (${failures[i].getProperty()})`;
        }
        throw new Error(`Invoke of '${tok}' failed: ${reasons}`);
    }
    let containsSecrets = false;
    const result = resp.getReturn();
    if (result === undefined) {
        return {
            result,
            containsSecrets,
        };
    }
    const properties = rpc_1.deserializeProperties(result);
    // Keep track of whether we need to mark the resulting output a secret.
    // and unwrap each individual value if it is a secret.
    for (const key of Object.keys(properties)) {
        containsSecrets = containsSecrets || rpc_1.isRpcSecret(properties[key]);
        properties[key] = rpc_1.unwrapRpcSecret(properties[key]);
    }
    return {
        result: properties,
        containsSecrets,
    };
}
/**
 * Helper function to call a method `tok`, which is offered by a provider plugin resource.
 *
 * May return a scalar as an object with a single key or a normal object.
 **/
function callAsync(tok, props, res, packageRef) {
    const label = `Calling function: tok=${tok}`;
    log.debug(label + (settings_1.excessiveDebugOutput ? `, props=${JSON.stringify(props)}` : ``));
    return debuggable_1.debuggablePromise(Promise.resolve().then(async () => {
        const done = settings_1.rpcKeepAlive();
        try {
            // Construct a provider reference from the given provider, if one is available on the resource.
            let provider = undefined;
            let version = undefined;
            let pluginDownloadURL = undefined;
            if (res) {
                if (res.__prov) {
                    provider = await resource_1.ProviderResource.register(res.__prov);
                }
                version = res.__version;
                pluginDownloadURL = res.__pluginDownloadURL;
            }
            const [serialized, propertyDepsResources] = await rpc_1.serializePropertiesReturnDeps(`call:${tok}`, props, {
                // We keep output values when serializing inputs for call.
                keepOutputValues: true,
                // We exclude resource references from 'argDependencies' when serializing inputs for call.
                // This way, component providers creating outputs for component inputs based on
                // 'argDependencies' won't create outputs for properties that only contain resource references.
                excludeResourceReferencesFromDependencies: true,
            });
            log.debug(`Call RPC prepared: tok=${tok}` + settings_1.excessiveDebugOutput ? `, obj=${JSON.stringify(serialized)}` : ``);
            const req = await createCallRequest(tok, serialized, propertyDepsResources, provider, version, pluginDownloadURL, packageRef);
            const monitor = settings_1.getMonitor();
            const resp = await debuggable_1.debuggablePromise(new Promise((innerResolve, innerReject) => {
                if (monitor === undefined) {
                    throw new Error("No monitor available");
                }
                monitor.call(req, (err, innerResponse) => {
                    log.debug(`Call RPC finished: tok=${tok}; err: ${err}, resp: ${innerResponse}`);
                    if (err) {
                        // If the monitor is unavailable, it is in the process of shutting down or has already
                        // shut down. Don't emit an error and don't do any more RPCs, just exit.
                        if (err.code === grpc.status.UNAVAILABLE || err.code === grpc.status.CANCELLED) {
                            settings_1.terminateRpcs();
                            err.message = "Resource monitor is terminating";
                            innerReject(err);
                            return;
                        }
                        // If the RPC failed, rethrow the error with a native exception and the message that
                        // the engine provided - it's suitable for user presentation.
                        innerReject(new Error(err.details));
                    }
                    else {
                        innerResolve(innerResponse);
                    }
                });
            }), label);
            // Deserialize the response and resolve the output.
            const { result, containsSecrets } = deserializeResponse(tok, resp);
            const deps = [];
            // Combine the individual dependencies into a single set of dependency resources.
            const rpcDeps = resp.getReturndependenciesMap();
            if (rpcDeps) {
                const urns = new Set();
                for (const [, returnDeps] of rpcDeps.entries()) {
                    for (const urn of returnDeps.getUrnsList()) {
                        urns.add(urn);
                    }
                }
                for (const urn of urns) {
                    deps.push(new resource_1.DependencyResource(urn));
                }
            }
            return {
                result: result,
                containsSecrets: containsSecrets,
                dependencies: deps,
                isKnown: true,
            };
        }
        finally {
            done();
        }
    }), label);
}
/**
 * Calls a method `tok` offered by a provider plugin resource.
 */
function call(tok, props, res, packageRef) {
    const [output, resolve] = createOutput(`call(${tok})`);
    callAsync(tok, props, res, packageRef)
        .then((response) => {
        const { result, isKnown, containsSecrets, dependencies } = response;
        // If the value the engine handed back is or contains an unknown value, the resolver will mark its value as
        // unknown automatically, so we just pass true for isKnown here. Note that unknown values will only be
        // present during previews (i.e. isDryRun() will be true).
        resolve(result, isKnown, containsSecrets, dependencies);
    })
        .catch((err) => {
        resolve(undefined, true, false, [], err);
    });
    return output;
}
exports.call = call;
/**
 * Calls a method `tok` offered by a provider plugin resource, but returns a single value.
 *
 * This method expects the result of `callAsync` to be a map containing a single value,
 * which it unwraps.
 */
function callSingle(tok, props, res, packageRef) {
    const [output, resolve] = createOutput(`callSingle(${tok})`);
    callAsync(tok, props, res, packageRef)
        .then((response) => {
        const { result, isKnown, containsSecrets, dependencies } = response;
        const value = extractSingleValue(result);
        resolve(value, isKnown, containsSecrets, dependencies, undefined);
    })
        .catch((err) => {
        resolve(undefined, true, false, [], err);
    });
    return output;
}
exports.callSingle = callSingle;
function createOutput(label) {
    let resolveValue;
    let rejectValue;
    let resolveIsKnown;
    let rejectIsKnown;
    let resolveIsSecret;
    let rejectIsSecret;
    let resolveDeps;
    let rejectDeps;
    const resolver = (v, isKnown, isSecret, deps = [], err) => {
        if (err) {
            rejectValue(err);
            rejectIsKnown(err);
            rejectIsSecret(err);
            rejectDeps(err);
        }
        else {
            resolveValue(v);
            resolveIsKnown(isKnown);
            resolveIsSecret(isSecret);
            resolveDeps(deps);
        }
    };
    const out = new output_1.Output([], debuggable_1.debuggablePromise(new Promise((resolve, reject) => {
        resolveValue = resolve;
        rejectValue = reject;
    }), `${label}Value`), debuggable_1.debuggablePromise(new Promise((resolve, reject) => {
        resolveIsKnown = resolve;
        rejectIsKnown = reject;
    }), `${label}IsKnown`), debuggable_1.debuggablePromise(new Promise((resolve, reject) => {
        resolveIsSecret = resolve;
        rejectIsSecret = reject;
    }), `${label}IsSecret`), debuggable_1.debuggablePromise(new Promise((resolve, reject) => {
        resolveDeps = resolve;
        rejectDeps = reject;
    }), `${label}Deps`));
    return [out, resolver];
}
async function createCallRequest(tok, serialized, serializedDeps, provider, version, pluginDownloadURL, packageRef) {
    if (provider !== undefined && typeof provider !== "string") {
        throw new Error("Incorrect provider type.");
    }
    const obj = gstruct.Struct.fromJavaScript(serialized);
    let packageRefStr = undefined;
    if (packageRef !== undefined) {
        packageRefStr = await packageRef;
        if (packageRefStr !== undefined) {
            // If we have a package reference we can clear some of the resource options
            version = undefined;
            pluginDownloadURL = undefined;
        }
    }
    const req = new resourceproto.ResourceCallRequest();
    req.setTok(tok);
    req.setArgs(obj);
    req.setProvider(provider || "");
    req.setVersion(version || "");
    req.setPlugindownloadurl(pluginDownloadURL || "");
    req.setPackageref(packageRefStr || "");
    const argDependencies = req.getArgdependenciesMap();
    for (const [key, propertyDeps] of serializedDeps) {
        const urns = new Set();
        for (const dep of propertyDeps) {
            const urn = await dep.urn.promise();
            urns.add(urn);
        }
        const deps = new resourceproto.ResourceCallRequest.ArgumentDependencies();
        deps.setUrnsList(Array.from(urns));
        argDependencies.set(key, deps);
    }
    return req;
}
//# sourceMappingURL=invoke.js.map