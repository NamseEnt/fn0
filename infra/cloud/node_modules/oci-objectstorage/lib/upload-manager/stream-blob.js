"use strict";
/**
 * Copyright (c) 2020, 2021 Oracle and/or its affiliates.  All rights reserved.
 * This software is dual-licensed to you under the Universal Permissive License (UPL) 1.0 as shown at https://oss.oracle.com/licenses/upl or Apache License 2.0 as shown at http://www.apache.org/licenses/LICENSE-2.0. You may choose either license.
 */
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.StreamBlob = void 0;
const oci_common_1 = require("oci-common");
class StreamBlob {
    constructor(stream, partSize) {
        this.partSize = partSize;
        this.streamBody = stream;
        this.size = oci_common_1.byteLength(stream);
    }
    /*
     * @Summary Get Data for the given Stream Body
     * returns
     */
    getData() {
        return __awaiter(this, void 0, void 0, function* () {
            return this.streamBody;
        });
    }
    /*
     * Get MD5 hash of the NodeFSBlob (NOT USED)
     * @return Md5 hash string
     */
    getMD5Hash() {
        return __awaiter(this, void 0, void 0, function* () {
            return "";
        });
    }
}
exports.StreamBlob = StreamBlob;
//# sourceMappingURL=stream-blob.js.map