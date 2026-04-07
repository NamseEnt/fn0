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
const async_hooks_1 = require("async_hooks");
const config = __importStar(require("./config"));
const nodeEnvKeys = {
    project: "PULUMI_NODEJS_PROJECT",
    rootDirectory: "PULUMI_NODEJS_ROOT_DIRECTORY",
    stack: "PULUMI_NODEJS_STACK",
    dryRun: "PULUMI_NODEJS_DRY_RUN",
    parallel: "PULUMI_NODEJS_PARALLEL",
    monitorAddr: "PULUMI_NODEJS_MONITOR",
    engineAddr: "PULUMI_NODEJS_ENGINE",
    syncDir: "PULUMI_NODEJS_SYNC",
    // Unlike the values above, this value is not set by the CLI and is
    // controlled via a user-set environment variable.
    cacheDynamicProviders: "PULUMI_NODEJS_CACHE_DYNAMIC_PROVIDERS",
    organization: "PULUMI_NODEJS_ORGANIZATION",
};
const pulumiEnvKeys = {
    legacyApply: "PULUMI_ENABLE_LEGACY_APPLY",
};
/**
 * @internal
 */
class LocalStore {
    constructor() {
        this.settings = {
            options: {
                organization: process.env[nodeEnvKeys.organization],
                project: process.env[nodeEnvKeys.project] || "project",
                rootDirectory: process.env[nodeEnvKeys.rootDirectory] || "rootDirectory",
                stack: process.env[nodeEnvKeys.stack] || "stack",
                dryRun: process.env[nodeEnvKeys.dryRun] === "true",
                monitorAddr: process.env[nodeEnvKeys.monitorAddr],
                engineAddr: process.env[nodeEnvKeys.engineAddr],
                syncDir: process.env[nodeEnvKeys.syncDir],
                cacheDynamicProviders: process.env[nodeEnvKeys.cacheDynamicProviders] !== "false",
                legacyApply: process.env[pulumiEnvKeys.legacyApply] === "true",
                maximumProcessListeners: 30,
            },
            rpcDone: Promise.resolve(),
            featureSupport: {},
        };
        this.config = {
            [config.configEnvKey]: process.env[config.configEnvKey] || "",
            [config.configSecretKeysEnvKey]: process.env[config.configSecretKeysEnvKey] || "",
        };
        this.stackResource = undefined;
        /**
         * Tracks the list of potential leak candidates.
         */
        this.leakCandidates = new Set();
        this.logErrorCount = 0;
        /* Tracks whether the monitor was terminated while we were waiting for an operation to complete */
        this.terminated = false;
        this.supportsSecrets = false;
        this.supportsResourceReferences = false;
        this.supportsOutputValues = false;
        this.supportsDeletedWith = false;
        this.supportsReplaceWith = false;
        this.supportsAliasSpecs = false;
        this.supportsTransforms = false;
        this.supportsInvokeTransforms = false;
        this.supportsParameterization = false;
        this.supportsResourceHooks = false;
        this.resourcePackages = new Map();
        this.resourceModules = new Map();
    }
}
exports.LocalStore = LocalStore;
/**
 * Get the root stack resource for the current stack deployment.
 *
 * @internal
 */
function getStackResource() {
    const { stackResource } = exports.getStore();
    return stackResource;
}
exports.getStackResource = getStackResource;
/**
 * Get the resource package map for the current stack deployment.
 *
 * @internal
 */
function getResourcePackages() {
    const store = exports.getGlobalStore();
    if (store.resourcePackages === undefined) {
        // resourcePackages can be undefined if an older SDK where it was not defined is created it.
        // In this case, we should initialize it to an empty map.
        store.resourcePackages = new Map();
    }
    return store.resourcePackages;
}
exports.getResourcePackages = getResourcePackages;
/**
 * Get the resource module map for the current stack deployment.
 *
 * @internal
 */
function getResourceModules() {
    const store = exports.getGlobalStore();
    if (store.resourceModules === undefined) {
        // resourceModules can be undefined if an older SDK where it was not defined is created it.
        // In this case, we should initialize it to an empty map.
        store.resourceModules = new Map();
    }
    return store.resourceModules;
}
exports.getResourceModules = getResourceModules;
/**
 * @internal
 */
function setStackResource(newStackResource) {
    const localStore = exports.getStore();
    globalThis.stackResource = newStackResource;
    localStore.stackResource = newStackResource;
}
exports.setStackResource = setStackResource;
// Ensure there is a global.asyncLocalStorage if this is the first copy of `runtime` to
// load.
if (global.asyncLocalStorage === undefined) {
    global.asyncLocalStorage = new async_hooks_1.AsyncLocalStorage();
}
/**
 * @internal
 */
function withLocalStorage(callback, ...args) {
    const store = new LocalStore();
    return global.asyncLocalStorage.run(store, callback, ...args);
}
exports.withLocalStorage = withLocalStorage;
/**
 * @internal
 */
function getLocalStore() {
    return global.asyncLocalStorage.getStore();
}
exports.getLocalStore = getLocalStore;
getLocalStore.captureReplacement = () => {
    const returnFunc = () => {
        if (global.globalStore === undefined) {
            global.globalStore = new LocalStore();
        }
        return global.globalStore;
    };
    return returnFunc;
};
/**
 * @internal
 */
exports.getStore = () => {
    const localStore = getLocalStore();
    if (localStore === undefined) {
        if (global.globalStore === undefined) {
            global.globalStore = new LocalStore();
        }
        return global.globalStore;
    }
    return localStore;
};
exports.getStore.captureReplacement = () => {
    const returnFunc = () => {
        if (global.globalStore === undefined) {
            global.globalStore = new LocalStore();
        }
        return global.globalStore;
    };
    return returnFunc;
};
/**
 * @internal
 */
exports.getGlobalStore = () => {
    if (global.globalStore === undefined) {
        global.globalStore = new LocalStore();
    }
    return global.globalStore;
};
//# sourceMappingURL=state.js.map