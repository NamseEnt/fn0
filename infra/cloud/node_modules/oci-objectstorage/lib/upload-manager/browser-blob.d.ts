/**
 * Copyright (c) 2020, 2021 Oracle and/or its affiliates.  All rights reserved.
 * This software is dual-licensed to you under the Universal Permissive License (UPL) 1.0 as shown at https://oss.oracle.com/licenses/upl or Apache License 2.0 as shown at http://www.apache.org/licenses/LICENSE-2.0. You may choose either license.
 */
import { UploadableBlob } from "./uploadable-blob";
export declare class BrowserBlob implements UploadableBlob {
    size: number;
    type: string;
    private readonly chunkSize;
    private readonly blob;
    constructor(blob: Blob, chunkSize: number);
    getData(): Promise<Blob>;
    text(): Promise<string>;
    arrayBuffer(): Promise<ArrayBuffer>;
    stream(): ReadableStream<any>;
    getMD5Hash(): Promise<string>;
    slice(start?: number, end?: number, contentType?: string): UploadableBlob;
}
