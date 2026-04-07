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
const utils = __importStar(require("../utils"));
/**
 * {@link Asset} represents a single blob of text or data that is managed as a
 * first-class entity.
 */
class Asset {
    constructor() {
        /**
         * A private field to help with RTTI that works in SxS scenarios.
         *
         * @internal
         */
        // eslint-disable-next-line @typescript-eslint/naming-convention,no-underscore-dangle,id-blacklist,id-match
        this.__pulumiAsset = true;
    }
    /**
     * Returns true if the given object is an {@link Asset}. This is designed to
     * work even when multiple copies of the Pulumi SDK have been loaded into
     * the same process.
     */
    static isInstance(obj) {
        return utils.isInstance(obj, "__pulumiAsset");
    }
}
exports.Asset = Asset;
/**
 * {@link FileAsset} is a kind of asset produced from a given path to a file on
 * the local filesystem.
 */
class FileAsset extends Asset {
    constructor(path) {
        super();
        this.path = Promise.resolve(path);
    }
}
exports.FileAsset = FileAsset;
/**
 * {@link StringAsset} is a kind of asset produced from an in-memory
 * UTF8-encoded string.
 */
class StringAsset extends Asset {
    constructor(text) {
        super();
        this.text = Promise.resolve(text);
    }
}
exports.StringAsset = StringAsset;
/**
 * {@link RemoteAsset} is a kind of asset produced from a given URI string. The
 * URI's scheme dictates the protocol for fetching contents: `file://` specifies
 * a local file, `http://` and `https://` specify HTTP and HTTPS, respectively.
 * Note that specific providers may recognize alternative schemes; this is
 * merely the smallest set that all providers support.
 */
class RemoteAsset extends Asset {
    constructor(uri) {
        super();
        this.uri = Promise.resolve(uri);
    }
}
exports.RemoteAsset = RemoteAsset;
//# sourceMappingURL=asset.js.map