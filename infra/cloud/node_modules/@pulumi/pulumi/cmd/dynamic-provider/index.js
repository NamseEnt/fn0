"use strict";
// Copyright 2016-2022, Pulumi Corporation.
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
// Enable source map support so we get good stack traces.
require("source-map-support/register");
const grpc = __importStar(require("@grpc/grpc-js"));
const metadata_1 = require("../../metadata");
const rpc = __importStar(require("../../runtime/rpc"));
const version_1 = require("../../version");
const dynConfig = __importStar(require("./config"));
const anyproto = __importStar(require("google-protobuf/google/protobuf/any_pb"));
const emptyproto = __importStar(require("google-protobuf/google/protobuf/empty_pb"));
const structproto = __importStar(require("google-protobuf/google/protobuf/struct_pb"));
const plugproto = __importStar(require("../../proto/plugin_pb"));
const provrpc = __importStar(require("../../proto/provider_grpc_pb"));
const provproto = __importStar(require("../../proto/provider_pb"));
const statusproto = __importStar(require("../../proto/status_pb"));
const runtime_1 = require("../../runtime");
const requireFromString = require("require-from-string");
const providerKey = "__provider";
// We track all uncaught errors here.  If we have any, we will make sure we always have a non-0 exit
// code.
const uncaughtErrors = new Set();
const uncaughtHandler = (err) => {
    if (!uncaughtErrors.has(err)) {
        uncaughtErrors.add(err);
        console.error(err.stack || err.message || "" + err);
    }
};
process.on("uncaughtException", uncaughtHandler);
// @ts-ignore 'unhandledRejection' will almost always invoke uncaughtHandler with an Error. so just
// suppress the TS strictness here.
process.on("unhandledRejection", uncaughtHandler);
process.on("exit", (code) => {
    // If there were any uncaught errors at all, we always want to exit with an error code.
    if (code === 0 && uncaughtErrors.size > 0) {
        process.exitCode = 1;
    }
});
const providerCache = {};
/**
 * getProvider deserializes the provider from the string found in
 * `props[providerKey]` and calls `provider.configure` with the config. The
 * deserialized and configured provider is stored in `providerCache`. This
 * guarantees that the provider is only deserialized and configured once per
 * process.
 */
async function getProvider(props, rawConfig) {
    const providerString = props[providerKey];
    let provider = providerCache[providerString];
    if (!provider) {
        provider = requireFromString(providerString).handler();
        providerCache[providerString] = provider;
        if (provider.configure) {
            const config = new dynConfig.Config(rawConfig, metadata_1.getProject());
            const req = { config };
            await provider.configure(req);
        }
    }
    // TODO[pulumi/pulumi#414]: investigate replacing requireFromString with eval
    return provider;
}
// Each of the *RPC functions below implements a single method of the resource provider gRPC interface. The CRUD
// functions--checkRPC, diffRPC, createRPC, updateRPC, and deleteRPC--all operate in a similar fashion:
//     1. Deserialize the dyanmic provider for the resource on which the function is operating
//     2. Call the dynamic provider's corresponding {check,diff,create,update,delete} method
//     3. Convert and return the results
// In all cases, the dynamic provider is available in its serialized form as a property of the resource;
// getProvider` is responsible for handling its deserialization. In the case of diffRPC, if the provider itself
// has changed, `diff` reports that the resource requires replacement and does not delegate to the dynamic provider.
// This allows the creation of the replacement resource to use the new provider while the deletion of the old
// resource uses the provider with which it was created.
class ResourceProviderService {
    cancel(call, callback) {
        callback(undefined, new emptyproto.Empty());
    }
    async configure(call, callback) {
        const protoArgs = call.request.getArgs();
        if (!protoArgs) {
            throw new Error("ConfigureRequest missing args");
        }
        const args = protoArgs.toJavaScript();
        const config = {};
        for (const [k, v] of Object.entries(args)) {
            if (k === providerKey) {
                continue;
            }
            config[k] = rpc.unwrapRpcSecret(v);
        }
        this.config = config;
        const resp = new provproto.ConfigureResponse();
        resp.setAcceptsecrets(false);
        callback(undefined, resp);
    }
    async invoke(call, callback) {
        const req = call.request;
        // TODO[pulumi/pulumi#406]: implement this.
        callback(new Error(`unknown function ${req.getTok()}`), undefined);
    }
    async check(call, callback) {
        try {
            const req = call.request;
            const resp = new provproto.CheckResponse();
            const olds = req.getOlds().toJavaScript();
            const news = req.getNews().toJavaScript();
            const provider = await getProvider(news[providerKey] === rpc.unknownValue ? olds : news, this.config);
            let inputs = news;
            let failures = [];
            if (provider.check) {
                const result = await provider.check(olds, news);
                if (result.inputs) {
                    inputs = result.inputs;
                }
                if (result.failures) {
                    failures = result.failures;
                }
            }
            else {
                // If no check method was provided, propagate the new inputs as-is.
                inputs = news;
            }
            inputs[providerKey] = news[providerKey];
            resp.setInputs(structproto.Struct.fromJavaScript(inputs));
            if (failures.length !== 0) {
                const failureList = [];
                for (const f of failures) {
                    const failure = new provproto.CheckFailure();
                    failure.setProperty(f.property);
                    failure.setReason(f.reason);
                    failureList.push(failure);
                }
                resp.setFailuresList(failureList);
            }
            callback(undefined, resp);
        }
        catch (e) {
            console.error(`${e}: ${e.stack}`);
            callback(e, undefined);
        }
    }
    checkConfig(call, callback) {
        callback({
            code: grpc.status.UNIMPLEMENTED,
            details: "CheckConfig is not implemented by the dynamic provider",
        }, undefined);
    }
    async diff(call, callback) {
        try {
            const req = call.request;
            const resp = new provproto.DiffResponse();
            // Note that we do not take any special action if the provider has changed. This allows a user to iterate on a
            // dynamic provider's implementation. This does require some care on the part of the user: each iteration of a
            // dynamic provider's implementation must be able to handle all state produced by prior iterations.
            //
            // Prior versions of the dynamic provider required that a dynamic resource be replaced any time its provider
            // implementation changed. This made iteration painful, especially if the dynamic resource was managing a
            // physical resource--in this case, the physical resource would be unnecessarily deleted and recreated each
            // time the provider was updated.
            const olds = req.getOlds().toJavaScript();
            const news = req.getNews().toJavaScript();
            const provider = await getProvider(news[providerKey] === rpc.unknownValue ? olds : news, this.config);
            if (provider.diff) {
                const result = await provider.diff(req.getId(), olds, news);
                if (result.changes === true) {
                    resp.setChanges(provproto.DiffResponse.DiffChanges.DIFF_SOME);
                }
                else if (result.changes === false) {
                    resp.setChanges(provproto.DiffResponse.DiffChanges.DIFF_NONE);
                }
                else {
                    resp.setChanges(provproto.DiffResponse.DiffChanges.DIFF_UNKNOWN);
                }
                if (result.replaces && result.replaces.length !== 0) {
                    resp.setReplacesList(result.replaces);
                }
                if (result.deleteBeforeReplace) {
                    resp.setDeletebeforereplace(result.deleteBeforeReplace);
                }
            }
            callback(undefined, resp);
        }
        catch (e) {
            console.error(`${e}: ${e.stack}`);
            callback(e, undefined);
        }
    }
    diffConfig(call, callback) {
        callback({
            code: grpc.status.UNIMPLEMENTED,
            details: "DiffConfig is not implemented by the dynamic provider",
        }, undefined);
    }
    async create(call, callback) {
        try {
            const req = call.request;
            const resp = new provproto.CreateResponse();
            const props = req.getProperties().toJavaScript();
            const provider = await getProvider(props, this.config);
            const result = await provider.create(props);
            const resultProps = resultIncludingProvider(result.outs, props);
            resp.setId(result.id);
            resp.setProperties(structproto.Struct.fromJavaScript(resultProps));
            callback(undefined, resp);
        }
        catch (e) {
            const response = grpcResponseFromError(e);
            return callback(/*err:*/ response, /*value:*/ null, /*metadata:*/ response.metadata);
        }
    }
    async read(call, callback) {
        try {
            const req = call.request;
            const resp = new provproto.ReadResponse();
            const id = req.getId();
            const props = req.getProperties().toJavaScript();
            const provider = await getProvider(props, this.config);
            if (provider.read) {
                // If there's a read function, consult the provider. Ensure to propagate the special __provider
                // value too, so that the provider's CRUD operations continue to function after a refresh.
                const result = await provider.read(id, props);
                resp.setId(result.id);
                const resultProps = resultIncludingProvider(result.props, props);
                resp.setProperties(structproto.Struct.fromJavaScript(resultProps));
            }
            else {
                // In the event of a missing read, simply return back the input state.
                resp.setId(id);
                resp.setProperties(req.getProperties());
            }
            callback(undefined, resp);
        }
        catch (e) {
            console.error(`${e}: ${e.stack}`);
            callback(e, undefined);
        }
    }
    async update(call, callback) {
        try {
            const req = call.request;
            const resp = new provproto.UpdateResponse();
            const olds = req.getOlds().toJavaScript();
            const news = req.getNews().toJavaScript();
            let result = {};
            const provider = await getProvider(news, this.config);
            if (provider.update) {
                result = (await provider.update(req.getId(), olds, news)) || {};
            }
            const resultProps = resultIncludingProvider(result.outs, news);
            resp.setProperties(structproto.Struct.fromJavaScript(resultProps));
            callback(undefined, resp);
        }
        catch (e) {
            const response = grpcResponseFromError(e);
            return callback(/*err:*/ response, /*value:*/ null, /*metadata:*/ response.metadata);
        }
    }
    async delete(call, callback) {
        try {
            const req = call.request;
            const props = req.getProperties().toJavaScript();
            const provider = await getProvider(props, this.config);
            if (provider.delete) {
                await provider.delete(req.getId(), props);
            }
            callback(undefined, new emptyproto.Empty());
        }
        catch (e) {
            console.error(`${e}: ${e.stack}`);
            callback(e, undefined);
        }
    }
    async getPluginInfo(call, callback) {
        const resp = new plugproto.PluginInfo();
        resp.setVersion(version_1.version);
        callback(undefined, resp);
    }
    handshake(call, callback) {
        callback({
            code: grpc.status.UNIMPLEMENTED,
            details: "Handshake is not implemented by the dynamic provider",
        }, undefined);
    }
    getSchema(call, callback) {
        callback({
            code: grpc.status.UNIMPLEMENTED,
            details: "GetSchema is not implemented by the dynamic provider",
        }, undefined);
    }
    construct(call, callback) {
        callback({
            code: grpc.status.UNIMPLEMENTED,
            details: "Construct is not implemented by the dynamic provider",
        }, undefined);
    }
    parameterize(call, callback) {
        callback({
            code: grpc.status.UNIMPLEMENTED,
            details: "Parameterize is not implemented by the dynamic provider",
        }, undefined);
    }
    call(call, callback) {
        callback({
            code: grpc.status.UNIMPLEMENTED,
            details: "Call is not implemented by the dynamic provider",
        }, undefined);
    }
    attach(call, callback) {
        callback({
            code: grpc.status.UNIMPLEMENTED,
            details: "Attach is not implemented by the dynamic provider",
        }, undefined);
    }
    getMapping(call, callback) {
        callback({
            code: grpc.status.UNIMPLEMENTED,
            details: "GetMapping is not implemented by the dynamic provider",
        }, undefined);
    }
    getMappings(call, callback) {
        callback({
            code: grpc.status.UNIMPLEMENTED,
            details: "GetMappings is not implemented by the dynamic provider",
        }, undefined);
    }
}
function resultIncludingProvider(result, props) {
    return Object.assign(result || {}, {
        [providerKey]: props[providerKey],
    });
}
/**
 * grpcResponseFromError creates a gRPC response representing an error from a dynamic provider's
 * resource. This is typically either a creation error, in which the API server has (virtually)
 * rejected the resource, or an initialization error, where the API server has accepted the
 * resource, but it failed to initialize (e.g., the app code is continually crashing and the
 * resource has failed to become alive).
 */
function grpcResponseFromError(e) {
    // Create response object.
    const resp = new statusproto.Status();
    resp.setCode(grpc.status.UNKNOWN);
    resp.setMessage(e.message);
    const metadata = new grpc.Metadata();
    if (e.id) {
        // Object created successfully, but failed to initialize. Pack initialization failure into
        // details.
        const detail = new provproto.ErrorResourceInitFailed();
        detail.setId(e.id);
        detail.setProperties(structproto.Struct.fromJavaScript(e.properties || {}));
        detail.setReasonsList(e.reasons || []);
        const details = new anyproto.Any();
        details.pack(detail.serializeBinary(), "pulumirpc.ErrorResourceInitFailed");
        // Add details to metadata.
        resp.addDetails(details);
        // NOTE: `grpc-status-details-bin` is a magic field that allows us to send structured
        // protobuf data as an error back through gRPC. This notion of details is a first-class in
        // the Go gRPC implementation, and the nodejs implementation has not quite caught up to it,
        // which is why it's cumbersome here.
        metadata.add("grpc-status-details-bin", Buffer.from(resp.serializeBinary()));
    }
    return {
        code: grpc.status.UNKNOWN,
        message: e.message,
        metadata: metadata,
    };
}
/** @internal */
async function main(args) {
    // The program requires a single argument: the address of the RPC endpoint for the engine.  It
    // optionally also takes a second argument, a reference back to the engine, but this may be missing.
    if (args.length === 0) {
        console.error("fatal: Missing <engine> address");
        process.exit(-1);
    }
    // Finally connect up the gRPC client/server and listen for incoming requests.
    const server = new grpc.Server({
        ...runtime_1.grpcChannelOptions,
    });
    const resourceProvider = new ResourceProviderService();
    server.addService(provrpc.ResourceProviderService, resourceProvider);
    const port = await new Promise((resolve, reject) => {
        server.bindAsync(`127.0.0.1:0`, grpc.ServerCredentials.createInsecure(), (err, p) => {
            if (err) {
                reject(err);
            }
            else {
                resolve(p);
            }
        });
    });
    // Emit the address so the monitor can read it to connect.  The gRPC server will keep the message loop alive.
    // We explicitly convert the number to a string so that Node doesn't colorize the output.
    console.log(port.toString());
}
exports.main = main;
main(process.argv.slice(2));
//# sourceMappingURL=index.js.map