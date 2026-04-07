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
// This file exports metadata about the context in which a program is being run.
const settings = __importStar(require("./runtime/settings"));
/**
 * Returns the current organization name.
 */
function getOrganization() {
    return settings.getOrganization();
}
exports.getOrganization = getOrganization;
/**
 * Returns the current project name. Throws an exception if none is registered.
 */
function getProject() {
    return settings.getProject();
}
exports.getProject = getProject;
/**
 * Returns the current stack name. Throws an exception if none is registered.
 */
function getStack() {
    return settings.getStack();
}
exports.getStack = getStack;
/**
 * Returns the root directory of the current Pulumi project.
 */
function getRootDirectory() {
    return settings.getRootDirectory();
}
exports.getRootDirectory = getRootDirectory;
//# sourceMappingURL=metadata.js.map