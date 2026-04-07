import * as responses from "./response";
import { WorkRequestClient } from "./client";
import { WaiterConfiguration } from "oci-common";
export declare function waitForWorkRequest(config: WaiterConfiguration | undefined, client: WorkRequestClient, workRequestId: string): Promise<responses.GetWorkRequestResponse>;
