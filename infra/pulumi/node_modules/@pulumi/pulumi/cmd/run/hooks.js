"use strict";
// Copyright 2025-2025, Pulumi Corporation.
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
var _a;
Object.defineProperty(exports, "__esModule", { value: true });
// This file will be loaded by Node.js to setup TypeScript transpilation import
// hooks when we're running in automatic ESM mode.
const tsn = __importStar(require("ts-node"));
let options = null;
/**
 * Called by Node.js when the hooks are registered in `run.ts`.
 */
async function initialize(args) {
    options = args;
}
exports.initialize = initialize;
const makeHooks = () => {
    const service = tsn.register(options);
    // @ts-ignore we're using a version of ts-node that has ts-node/esm available.
    return tsn.createEsmHooks(service);
};
_a = makeHooks(), exports.resolve = _a.resolve, exports.load = _a.load, exports.getFormat = _a.getFormat, exports.transformSource = _a.transformSource;
//# sourceMappingURL=hooks.js.map