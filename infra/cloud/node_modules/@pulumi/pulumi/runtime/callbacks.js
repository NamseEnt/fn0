"use strict";
// Copyright 2016-2025, Pulumi Corporation.
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
const crypto_1 = require("crypto");
const gstruct = __importStar(require("google-protobuf/google/protobuf/struct_pb"));
const log = __importStar(require("../log"));
const output_1 = require("../output");
const callrpc = __importStar(require("../proto/callback_grpc_pb"));
const callback_pb_1 = require("../proto/callback_pb");
const resproto = __importStar(require("../proto/resource_pb"));
const resource_1 = require("../resource");
const resource_2 = require("./resource");
const rpc_1 = require("./rpc");
const debuggable_1 = require("./debuggable");
const settings_1 = require("./settings");
class CallbackServer {
    constructor(monitor) {
        this._callbacks = new Map();
        this._pendingRegistrations = 0;
        this._awaitQueue = [];
        this._monitor = monitor;
        this._server = new grpc.Server({
            ...settings_1.grpcChannelOptions,
        });
        const implementation = {
            invoke: this.invoke.bind(this),
        };
        this._server.addService(callrpc.CallbacksService, implementation);
        const self = this;
        this._target = new Promise((resolve, reject) => {
            self._server.bindAsync(`127.0.0.1:0`, grpc.ServerCredentials.createInsecure(), (err, port) => {
                if (err !== null) {
                    reject(err);
                    return;
                }
                // The running callbacks server keeps the Node.js eventloop
                // active, so we will never exit the process. Normally we would
                // shutdown the server when we're done, however we don't have a
                // good way to tell when that is the case. Consider a user
                // provider Pulumi program:
                //
                // setTimeout(() => new random.RandomPet(
                //      "username", {}, { transforms: [...] }), 2000);
                //
                // When we import the user program (module really), it will
                // return immediately, but we have to wait for the timeout to
                // trigger and then the resource registration to complete.
                //
                // Usually the best indication that no more work is outstanding
                // is `process.on("beforeExit", ...)`, which will be emitted
                // when there is no more work waiting in the eventloop. However
                // if we have a server running, we'll never get to that event.
                //
                // To break out of this catch-22, we `unref` the server, and the
                // sessions created by the server.
                //
                // https://nodejs.org/api/net.html#serverunref
                // https://nodejs.org/api/http2.html#http2sessionunref
                try {
                    // https://github.com/grpc/grpc-node/blob/01db2bc6206963a6678ee05fa4651fd2ee40068c/packages/grpc-js/src/server.ts#L271
                    // @ts-ignore 2341 // http2Servers is private
                    const httpServer = Array.from(self._server.http2Servers.keys())[0];
                    if (httpServer) {
                        httpServer.unref();
                        httpServer.on("session", (session) => {
                            session.unref();
                        });
                    }
                }
                catch (error) {
                    log.debug(`failed to unref the callback server: ${error}`);
                }
                // The server takes a while to _actually_ startup so we need to keep trying to send an invoke
                // to ourselves before we resolve the address to tell the engine about it.
                const target = `127.0.0.1:${port}`;
                const client = new callrpc.CallbacksClient(target, grpc.credentials.createInsecure());
                const connect = () => {
                    client.invoke(new callback_pb_1.CallbackInvokeRequest(), (error, _) => {
                        if (error?.code === grpc.status.UNAVAILABLE) {
                            setTimeout(connect, 1000);
                            return;
                        }
                        // The expected error given we didn't give a token to the invoke.
                        if (error?.details === "callback not found: ") {
                            resolve(target);
                            return;
                        }
                        reject(error);
                    });
                };
                connect();
            });
        });
    }
    awaitStackRegistrations() {
        if (this._pendingRegistrations === 0) {
            return Promise.resolve();
        }
        return new Promise((resolve, reject) => {
            this._awaitQueue.push((reason) => {
                if (reason !== undefined) {
                    reject(reason);
                }
                else {
                    resolve();
                }
            });
        });
    }
    shutdown() {
        this._server.forceShutdown();
    }
    async invoke(call, callback) {
        const req = call.request;
        const cb = this._callbacks.get(req.getToken());
        if (cb === undefined) {
            const err = new grpc.StatusBuilder();
            err.withCode(grpc.status.INVALID_ARGUMENT);
            err.withDetails("callback not found: " + req.getToken());
            callback(err.build());
            return;
        }
        try {
            const response = await cb(req.getRequest_asU8());
            const resp = new callback_pb_1.CallbackInvokeResponse();
            resp.setResponse(response.serializeBinary());
            callback(null, resp);
        }
        catch (e) {
            const err = new grpc.StatusBuilder();
            err.withCode(grpc.status.UNKNOWN);
            if (e instanceof Error) {
                err.withDetails(e.message);
            }
            else {
                err.withDetails(JSON.stringify(e));
            }
            callback(err.build());
        }
    }
    async registerTransform(transform) {
        const cb = async (bytes) => {
            const request = resproto.TransformRequest.deserializeBinary(bytes);
            let opts = request.getOptions() || new resproto.TransformResourceOptions();
            let ropts;
            if (request.getCustom()) {
                ropts = {
                    deleteBeforeReplace: opts.getDeleteBeforeReplace(),
                    additionalSecretOutputs: opts.getAdditionalSecretOutputsList(),
                    import: opts.getImport(),
                };
            }
            else {
                const providers = {};
                for (const [key, value] of opts.getProvidersMap().entries()) {
                    providers[key] = new resource_1.DependencyProviderResource(value);
                }
                ropts = {
                    providers: providers,
                };
            }
            ropts.aliases = opts.getAliasesList().map((alias) => {
                if (alias.hasUrn()) {
                    return alias.getUrn();
                }
                else {
                    const spec = alias.getSpec();
                    if (spec === undefined) {
                        throw new Error("alias must have either a urn or a spec");
                    }
                    const nodeAlias = {
                        name: spec.getName(),
                        type: spec.getType(),
                        project: spec.getProject(),
                        stack: spec.getStack(),
                        parent: spec.getParenturn() !== "" ? new resource_1.DependencyResource(spec.getParenturn()) : undefined,
                    };
                    if (spec.getNoparent()) {
                        nodeAlias.parent = resource_1.rootStackResource;
                    }
                    return nodeAlias;
                }
            });
            const timeouts = opts.getCustomTimeouts();
            if (timeouts !== undefined) {
                ropts.customTimeouts = {
                    create: timeouts.getCreate(),
                    update: timeouts.getUpdate(),
                    delete: timeouts.getDelete(),
                };
            }
            ropts.hooks = resource_2.hookBindingFromProto(opts.getHooks());
            ropts.deletedWith =
                opts.getDeletedWith() !== "" ? new resource_1.DependencyResource(opts.getDeletedWith()) : undefined;
            ropts.replaceWith = opts.getReplaceWithList().map((dep) => new resource_1.DependencyResource(dep));
            ropts.dependsOn = opts.getDependsOnList().map((dep) => new resource_1.DependencyResource(dep));
            ropts.ignoreChanges = opts.getIgnoreChangesList();
            ropts.parent = request.getParent() !== "" ? new resource_1.DependencyResource(request.getParent()) : undefined;
            ropts.pluginDownloadURL = opts.getPluginDownloadUrl() !== "" ? opts.getPluginDownloadUrl() : undefined;
            ropts.protect = opts.getProtect();
            ropts.provider = opts.getProvider() !== "" ? new resource_1.DependencyProviderResource(opts.getProvider()) : undefined;
            ropts.replaceOnChanges = opts.getReplaceOnChangesList();
            ropts.retainOnDelete = opts.getRetainOnDelete();
            ropts.version = opts.getVersion() !== "" ? opts.getVersion() : undefined;
            const props = request.getProperties();
            const args = {
                custom: request.getCustom(),
                type: request.getType(),
                name: request.getName(),
                props: props === undefined ? {} : rpc_1.deserializeProperties(props),
                opts: ropts,
            };
            const result = await transform(args);
            const response = new resproto.TransformResponse();
            if (result === undefined) {
                response.setProperties(request.getProperties());
                response.setOptions(request.getOptions());
            }
            else {
                const mprops = await rpc_1.serializeProperties("props", result.props);
                response.setProperties(gstruct.Struct.fromJavaScript(mprops));
                // Copy the options over.
                if (result.opts !== undefined) {
                    opts = new resproto.TransformResourceOptions();
                    if (result.opts.aliases !== undefined) {
                        const aliases = [];
                        const uniqueAliases = new Set();
                        for (const alias of result.opts.aliases || []) {
                            const aliasVal = await output_1.output(alias).promise();
                            if (!uniqueAliases.has(aliasVal)) {
                                uniqueAliases.add(aliasVal);
                                aliases.push(aliasVal);
                            }
                        }
                        opts.setAliasesList(await resource_2.mapAliasesForRequest(aliases, request.getParent()));
                    }
                    if (result.opts.customTimeouts !== undefined) {
                        const customTimeouts = new resproto.RegisterResourceRequest.CustomTimeouts();
                        if (result.opts.customTimeouts.create !== undefined) {
                            customTimeouts.setCreate(result.opts.customTimeouts.create);
                        }
                        if (result.opts.customTimeouts.update !== undefined) {
                            customTimeouts.setUpdate(result.opts.customTimeouts.update);
                        }
                        if (result.opts.customTimeouts.delete !== undefined) {
                            customTimeouts.setDelete(result.opts.customTimeouts.delete);
                        }
                        opts.setCustomTimeouts(customTimeouts);
                    }
                    if (result.opts.deletedWith !== undefined) {
                        opts.setDeletedWith(await result.opts.deletedWith.urn.promise());
                    }
                    if (result.opts.replaceWith !== undefined) {
                        opts.setReplaceWithList(await Promise.all(result.opts.replaceWith.map(async (dep) => await dep.urn.promise())));
                    }
                    if (result.opts.dependsOn !== undefined) {
                        const resolvedDeps = await output_1.output(result.opts.dependsOn).promise();
                        const deps = [];
                        if (resource_1.Resource.isInstance(resolvedDeps)) {
                            deps.push(await resolvedDeps.urn.promise());
                        }
                        else {
                            for (const dep of resolvedDeps) {
                                deps.push(await dep.urn.promise());
                            }
                        }
                        opts.setDependsOnList(deps);
                    }
                    if (result.opts.ignoreChanges !== undefined) {
                        opts.setIgnoreChangesList(result.opts.ignoreChanges);
                    }
                    if (result.opts.pluginDownloadURL !== undefined) {
                        opts.setPluginDownloadUrl(result.opts.pluginDownloadURL);
                    }
                    if (result.opts.protect !== undefined) {
                        opts.setProtect(result.opts.protect);
                    }
                    if (result.opts.provider !== undefined) {
                        const providerURN = await result.opts.provider.urn.promise();
                        const providerID = (await result.opts.provider.id.promise()) || rpc_1.unknownValue;
                        opts.setProvider(`${providerURN}::${providerID}`);
                    }
                    if (result.opts.replaceOnChanges !== undefined) {
                        opts.setReplaceOnChangesList(result.opts.replaceOnChanges);
                    }
                    if (result.opts.retainOnDelete !== undefined) {
                        opts.setRetainOnDelete(result.opts.retainOnDelete);
                    }
                    if (result.opts.version !== undefined) {
                        opts.setVersion(result.opts.version);
                    }
                    if (result.opts.hooks !== undefined) {
                        opts.setHooks(await resource_2.prepareHooks(result.opts.hooks, request.getName()));
                    }
                    if (request.getCustom()) {
                        const copts = result.opts;
                        if (copts.deleteBeforeReplace !== undefined) {
                            opts.setDeleteBeforeReplace(copts.deleteBeforeReplace);
                        }
                        if (copts.additionalSecretOutputs !== undefined) {
                            opts.setAdditionalSecretOutputsList(copts.additionalSecretOutputs);
                        }
                        if (copts.import !== undefined) {
                            opts.setImport(copts.import);
                        }
                    }
                    else {
                        const copts = result.opts;
                        if (copts.providers !== undefined) {
                            const providers = opts.getProvidersMap();
                            if (copts.providers && !Array.isArray(copts.providers)) {
                                for (const k in copts.providers) {
                                    if (Object.prototype.hasOwnProperty.call(copts.providers, k)) {
                                        const v = copts.providers[k];
                                        if (k !== v.getPackage()) {
                                            const message = `provider resource map where key ${k} doesn't match provider ${v.getPackage()}`;
                                            log.warn(message);
                                        }
                                    }
                                }
                            }
                            const provs = Object.values(copts.providers);
                            for (const prov of provs) {
                                const providerURN = await prov.urn.promise();
                                const providerID = (await prov.id.promise()) || rpc_1.unknownValue;
                                providers.set(prov.getPackage(), `${providerURN}::${providerID}`);
                            }
                            opts.clearProvidersMap();
                        }
                    }
                }
                response.setOptions(opts);
            }
            return response;
        };
        const tryCb = async (bytes) => {
            try {
                return await cb(bytes);
            }
            catch (e) {
                throw new Error(`transform failed: ${e}`);
            }
        };
        const uuid = crypto_1.randomUUID();
        this._callbacks.set(uuid, tryCb);
        const req = new callback_pb_1.Callback();
        req.setToken(uuid);
        req.setTarget(await this._target);
        return req;
    }
    registerStackTransform(transform) {
        this._pendingRegistrations++;
        this.registerTransform(transform)
            .then((req) => {
            return new Promise((resolve, reject) => {
                this._monitor.registerStackTransform(req, (err, _) => {
                    if (err !== null) {
                        // Remove this from the list of callbacks given we didn't manage to actually register it.
                        this._callbacks.delete(req.getToken());
                        reject();
                    }
                    else {
                        resolve();
                    }
                });
            });
        }, (err) => log.error(`failed to register stack transform: ${err}`))
            .finally(() => {
            this._pendingRegistrations--;
            if (this._pendingRegistrations === 0) {
                const queue = this._awaitQueue;
                this._awaitQueue = [];
                for (const waiter of queue) {
                    waiter();
                }
            }
        });
    }
    async registerStackInvokeTransformAsync(transform) {
        const cb = async (bytes) => {
            const request = resproto.TransformInvokeRequest.deserializeBinary(bytes);
            let opts = request.getOptions() || new resproto.TransformInvokeOptions();
            const ropts = {};
            ropts.pluginDownloadURL = opts.getPluginDownloadUrl() !== "" ? opts.getPluginDownloadUrl() : undefined;
            ropts.provider = opts.getProvider() !== "" ? new resource_1.DependencyProviderResource(opts.getProvider()) : undefined;
            ropts.version = opts.getVersion() !== "" ? opts.getVersion() : undefined;
            const invokeArgs = request.getArgs();
            const args = {
                token: request.getToken(),
                args: invokeArgs === undefined ? {} : rpc_1.deserializeProperties(invokeArgs),
                opts: ropts,
            };
            const result = await transform(args);
            const response = new resproto.TransformInvokeResponse();
            if (result === undefined) {
                response.setArgs(request.getArgs());
                response.setOptions(request.getOptions());
            }
            else {
                const margs = await rpc_1.serializeProperties("args", result.args);
                response.setArgs(gstruct.Struct.fromJavaScript(margs));
                // Copy the options over.
                if (result.opts !== undefined) {
                    opts = new resproto.TransformInvokeOptions();
                    if (result.opts.pluginDownloadURL !== undefined) {
                        opts.setPluginDownloadUrl(result.opts.pluginDownloadURL);
                    }
                    if (result.opts.provider !== undefined) {
                        const providerURN = await result.opts.provider.urn.promise();
                        const providerID = (await result.opts.provider.id.promise()) || rpc_1.unknownValue;
                        opts.setProvider(`${providerURN}::${providerID}`);
                    }
                    if (result.opts.version !== undefined) {
                        opts.setVersion(result.opts.version);
                    }
                    response.setOptions(opts);
                }
            }
            return response;
        };
        const tryCb = async (bytes) => {
            try {
                return await cb(bytes);
            }
            catch (e) {
                throw new Error(`transform failed: ${e}`);
            }
        };
        const uuid = crypto_1.randomUUID();
        this._callbacks.set(uuid, tryCb);
        const req = new callback_pb_1.Callback();
        req.setToken(uuid);
        req.setTarget(await this._target);
        return req;
    }
    registerStackInvokeTransform(transform) {
        this._pendingRegistrations++;
        this.registerStackInvokeTransformAsync(transform)
            .then((req) => {
            return new Promise((resolve, reject) => {
                this._monitor.registerStackInvokeTransform(req, (err, _) => {
                    if (err !== null) {
                        // Remove this from the list of callbacks given we didn't manage to actually register it.
                        this._callbacks.delete(req.getToken());
                        reject();
                    }
                    else {
                        resolve();
                    }
                });
            });
        }, (err) => log.error(`failed to register stack transform: ${err}`))
            .finally(() => {
            this._pendingRegistrations--;
            if (this._pendingRegistrations === 0) {
                const queue = this._awaitQueue;
                this._awaitQueue = [];
                for (const waiter of queue) {
                    waiter();
                }
            }
        });
    }
    async registerResourceHook(hook) {
        const cb = async (bytes) => {
            try {
                const request = resproto.ResourceHookRequest.deserializeBinary(bytes);
                const newInputs = request.getNewInputs();
                const oldInputs = request.getOldInputs();
                const newOutputs = request.getNewOutputs();
                const oldOutputs = request.getOldOutputs();
                await hook.callback({
                    urn: request.getUrn(),
                    id: request.getId(),
                    name: request.getName(),
                    type: request.getType(),
                    newInputs: newInputs
                        ? outputtifySecrets(rpc_1.deserializeProperties(newInputs, true /*keepUnknowns */))
                        : undefined,
                    oldInputs: oldInputs
                        ? outputtifySecrets(rpc_1.deserializeProperties(oldInputs, true /*keepUnknowns */))
                        : undefined,
                    newOutputs: newOutputs
                        ? outputtifySecrets(rpc_1.deserializeProperties(newOutputs, true /*keepUnknowns */))
                        : undefined,
                    oldOutputs: oldOutputs
                        ? outputtifySecrets(rpc_1.deserializeProperties(oldOutputs, true /*keepUnknowns */))
                        : undefined,
                });
            }
            catch (error) {
                const response = new resproto.ResourceHookResponse();
                response.setError(error.message);
                return response;
            }
            return new resproto.ResourceHookResponse();
        };
        const uuid = crypto_1.randomUUID();
        this._callbacks.set(uuid, cb);
        const callback = new callback_pb_1.Callback();
        callback.setToken(uuid);
        callback.setTarget(await this._target);
        const req = new resproto.RegisterResourceHookRequest();
        req.setCallback(callback);
        req.setName(hook.name);
        req.setOnDryRun(hook.opts?.onDryRun ?? false);
        const done = settings_1.rpcKeepAlive();
        return debuggable_1.debuggablePromise(new Promise((resolve, reject) => {
            this._monitor.registerResourceHook(req, (err, _) => {
                if (err !== null) {
                    // Remove this from the list of callbacks given we didn't manage to actually register it.
                    this._callbacks.delete(uuid);
                    reject(err);
                }
                else {
                    resolve();
                }
                done();
            });
        }), `resourceHook:${hook.name}`);
    }
}
exports.CallbackServer = CallbackServer;
function outputtifySecrets(props) {
    const outputs = {};
    for (const [key, value] of Object.entries(props)) {
        if (rpc_1.isRpcSecret(value)) {
            outputs[key] = output_1.secret(rpc_1.unwrapRpcSecret(value));
        }
        else {
            outputs[key] = value;
        }
    }
    return outputs;
}
//# sourceMappingURL=callbacks.js.map