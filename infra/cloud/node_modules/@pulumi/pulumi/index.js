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
function __export(m) {
    for (var p in m) if (!exports.hasOwnProperty(p)) exports[p] = m[p];
}
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
// Export top-level elements.
__export(require("./config"));
__export(require("./errors"));
__export(require("./metadata"));
__export(require("./output"));
__export(require("./resource"));
__export(require("./stackReference"));
__export(require("./stash"));
// Export submodules individually.
const asset = __importStar(require("./asset"));
exports.asset = asset;
const automation = __importStar(require("./automation"));
exports.automation = automation;
const dynamic = __importStar(require("./dynamic"));
exports.dynamic = dynamic;
const iterable = __importStar(require("./iterable"));
exports.iterable = iterable;
const log = __importStar(require("./log"));
exports.log = log;
const provider = __importStar(require("./provider"));
exports.provider = provider;
const runtime = __importStar(require("./runtime"));
exports.runtime = runtime;
const utils = __importStar(require("./utils"));
exports.utils = utils;
// @pulumi is a deployment-only module.  If someone tries to capture it, and we fail for some reason
// we want to give a good message about what the problem likely is.  Note that capturing a
// deployment time module can be ok in some cases.  For example, using "new pulumi.Config" is fine.
// However, in general, the majority of this API is not safe to use at 'run time' and will fail.
/** @internal */
exports.deploymentOnlyModule = true;
//# sourceMappingURL=index.js.map