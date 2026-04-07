"use strict";
// Copyright 2016-2018, Pulumi Corporation.
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
const rpc_1 = require("./rpc");
const settings_1 = require("./settings");
const state_1 = require("./state");
const structproto = __importStar(require("google-protobuf/google/protobuf/struct_pb"));
const provproto = __importStar(require("../proto/provider_pb"));
const resproto = __importStar(require("../proto/resource_pb"));
class MockMonitor {
    constructor(mocks) {
        this.mocks = mocks;
        this.resources = new Map();
    }
    newUrn(parent, type, name) {
        if (parent) {
            const qualifiedType = parent.split("::")[2];
            const parentType = qualifiedType.split("$").pop();
            type = parentType + "$" + type;
        }
        return "urn:pulumi:" + [settings_1.getStack(), settings_1.getProject(), type, name].join("::");
    }
    async invoke(req, callback) {
        try {
            const tok = req.getTok();
            const inputs = rpc_1.deserializeProperties(req.getArgs());
            if (tok === "pulumi:pulumi:getResource") {
                const registeredResource = this.resources.get(inputs.urn);
                if (!registeredResource) {
                    throw new Error(`unknown resource ${inputs.urn}`);
                }
                const resp = new provproto.InvokeResponse();
                resp.setReturn(structproto.Struct.fromJavaScript(registeredResource));
                callback(null, resp);
                return;
            }
            const result = await this.mocks.call({
                token: tok,
                inputs: inputs,
                provider: req.getProvider(),
            });
            const response = new provproto.InvokeResponse();
            response.setReturn(structproto.Struct.fromJavaScript(await rpc_1.serializeProperties("", result)));
            callback(null, response);
        }
        catch (err) {
            callback(err, undefined);
        }
    }
    async readResource(req, callback) {
        try {
            const result = await this.mocks.newResource({
                type: req.getType(),
                name: req.getName(),
                inputs: rpc_1.deserializeProperties(req.getProperties()),
                provider: req.getProvider(),
                custom: true,
                id: req.getId(),
            });
            const urn = this.newUrn(req.getParent(), req.getType(), req.getName());
            const serializedState = await rpc_1.serializeProperties("", result.state);
            this.resources.set(urn, { urn, id: result.id ?? null, state: serializedState });
            const response = new resproto.ReadResourceResponse();
            response.setUrn(urn);
            response.setProperties(structproto.Struct.fromJavaScript(serializedState));
            callback(null, response);
        }
        catch (err) {
            callback(err, undefined);
        }
    }
    async registerResource(req, callback) {
        try {
            const result = await this.mocks.newResource({
                type: req.getType(),
                name: req.getName(),
                inputs: rpc_1.deserializeProperties(req.getObject()),
                provider: req.getProvider(),
                custom: req.getCustom(),
                id: req.getImportid(),
            });
            const urn = this.newUrn(req.getParent(), req.getType(), req.getName());
            const serializedState = await rpc_1.serializeProperties("", result.state);
            this.resources.set(urn, { urn, id: result.id ?? null, state: serializedState });
            const response = new resproto.RegisterResourceResponse();
            response.setUrn(urn);
            response.setId(result.id || "");
            response.setObject(structproto.Struct.fromJavaScript(serializedState));
            callback(null, response);
        }
        catch (err) {
            callback(err, undefined);
        }
    }
    registerResourceOutputs(req, callback) {
        try {
            const registeredResource = this.resources.get(req.getUrn());
            if (!registeredResource) {
                throw new Error(`unknown resource ${req.getUrn()}`);
            }
            registeredResource.state = req.getOutputs();
            callback(null, {});
        }
        catch (err) {
            callback(err, undefined);
        }
    }
    supportsFeature(req, callback) {
        const id = req.getId();
        // Support for "outputValues" is deliberately disabled for the mock monitor so
        // instances of `Output` don't show up in `MockResourceArgs` inputs.
        const hasSupport = id !== "outputValues";
        callback(null, {
            getHassupport: () => hasSupport,
        });
    }
    registerPackage(req, callback) {
        // Mocks don't _really_ support packages, so we just return a fake package ref.
        const resp = new resproto.RegisterPackageResponse();
        resp.setRef("mock-uuid");
        callback(null, resp);
    }
}
exports.MockMonitor = MockMonitor;
/**
 * Configures the Pulumi runtime to use the given mocks for testing.
 *
 * @param mocks
 *  The mocks to use for calls to provider functions and resource construction.
 * @param project
 *  If provided, the name of the Pulumi project. Defaults to "project".
 * @param stack
 *  If provided, the name of the Pulumi stack. Defaults to "stack".
 * @param preview
 *  If provided, indicates whether or not the program is running a preview. Defaults to false.
 * @param organization
 *  If provided, the name of the Pulumi organization. Defaults to nothing.
 */
async function setMocks(mocks, project, stack, preview, organization) {
    settings_1.setMockOptions(new MockMonitor(mocks), project, stack, preview, organization);
    // Mocks enable all features except outputValues.
    const store = state_1.getStore();
    store.supportsSecrets = true;
    store.supportsResourceReferences = true;
    store.supportsOutputValues = false;
    store.supportsDeletedWith = true;
    store.supportsReplaceWith = true;
    store.supportsAliasSpecs = true;
    store.supportsTransforms = false;
    store.supportsParameterization = true;
    store.supportsResourceHooks = true;
}
exports.setMocks = setMocks;
//# sourceMappingURL=mocks.js.map