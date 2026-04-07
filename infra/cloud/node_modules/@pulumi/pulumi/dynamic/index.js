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
const output_1 = require("../output");
const resource = __importStar(require("../resource"));
const settings = __importStar(require("../runtime/settings"));
const providerCache = new WeakMap();
function serializeProvider(provider) {
    let result;
    // caching is enabled by default as of 3.0
    if (settings.cacheDynamicProviders()) {
        const cachedProvider = providerCache.get(provider);
        if (cachedProvider) {
            result = cachedProvider;
        }
        else {
            result = serializeFunctionMaybeSecret(provider);
            providerCache.set(provider, result);
        }
    }
    else {
        result = serializeFunctionMaybeSecret(provider);
    }
    return result;
}
function serializeFunctionMaybeSecret(provider) {
    // Load runtime/closure on demand, as its dependencies are slow to load.
    //
    // See https://www.typescriptlang.org/docs/handbook/modules.html#optional-module-loading-and-other-advanced-loading-scenarios
    const sc = require("../runtime/closure/serializeClosure");
    const sfPromise = sc.serializeFunction(() => provider, { allowSecrets: true });
    // Create an Output from the promise's text and containsSecrets properties.  Uses the internal API since we don't provide a public interface for this.
    return new output_1.Output([], sfPromise.then((sf) => sf.text), new Promise((resolve) => resolve(true)), sfPromise.then((sf) => sf.containsSecrets), new Promise((resolve) => resolve([])));
}
/**
 * {@link Resource} represents a Pulumi resource that incorporates an inline
 * implementation of the Resource's CRUD operations.
 */
class Resource extends resource.CustomResource {
    /**
     * Creates a new dynamic resource.
     *
     * @param provider
     *  The implementation of the resource's CRUD operations.
     * @param name
     *  The name of the resource.
     * @param props
     *  The arguments to use to populate the new resource. Must not define the
     *  reserved property "__provider".
     * @param opts
     *  A bag of options that control this resource's behavior.
     * @param module
     *  The module of the resource.
     * @param type
     *  The type of the resource.
     */
    constructor(provider, name, props, opts, module, type = "Resource") {
        const providerKey = "__provider";
        if (props[providerKey]) {
            throw new Error("A dynamic resource must not define the __provider key");
        }
        props[providerKey] = serializeProvider(provider);
        super(`pulumi-nodejs:dynamic${module ? `/${module}` : ""}:${type}`, name, props, opts);
    }
}
exports.Resource = Resource;
//# sourceMappingURL=index.js.map