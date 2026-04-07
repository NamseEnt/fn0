/**
 * Copyright (c) 2020, 2021 Oracle and/or its affiliates.  All rights reserved.
 * This software is dual-licensed to you under the Universal Permissive License (UPL) 1.0 as shown at https://oss.oracle.com/licenses/upl or Apache License 2.0 as shown at http://www.apache.org/licenses/LICENSE-2.0. You may choose either license.
 */
import { BinaryBody } from "./types";
import { UploadableStream } from "./uploadable-stream";
export declare class StreamBlob implements UploadableStream {
    partSize: number;
    streamBody: BinaryBody;
    size: number;
    constructor(stream: BinaryBody, partSize: number);
    getData(): Promise<BinaryBody>;
    getMD5Hash(): Promise<string>;
}
