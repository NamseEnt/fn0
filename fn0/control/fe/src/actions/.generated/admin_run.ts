// Auto-generated from src/actions/admin_run.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const NSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("PosInt"),
    v: z.number(),
  }),
    z.object({
    t: z.literal("NegInt"),
    v: z.number(),
  }),
    z.object({
    t: z.literal("Float"),
    v: z.number(),
  })
  ]);

const NumberSchema = z.object({
    n: NSchema,
  });

const MapSchema = z.object({
    map: z.record(z.string(), ValueSchema),
  });

const ValueSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Null"),
  }),
    z.object({
    t: z.literal("Bool"),
    v: z.boolean(),
  }),
    z.object({
    t: z.literal("Number"),
    v: NumberSchema,
  }),
    z.object({
    t: z.literal("String"),
    v: z.string(),
  }),
    z.object({
    t: z.literal("Array"),
    v: z.array(ValueSchema),
  }),
    z.object({
    t: z.literal("Object"),
    v: MapSchema,
  })
  ]);

const InputSchema = z.object({
    projectId: z.string(),
    task: z.string(),
    input: ValueSchema,
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    status: z.number(),
    contentType: z.string().optional(),
    body: z.string(),
  }),
    z.object({
    t: z.literal("NotLoggedIn"),
  }),
    z.object({
    t: z.literal("NotFound"),
  }),
    z.object({
    t: z.literal("Forbidden"),
  }),
    z.object({
    t: z.literal("NotDeployed"),
  }),
    z.object({
    t: z.literal("UpstreamError"),
    status: z.number(),
    contentType: z.string().optional(),
    body: z.string(),
  }),
    z.object({
    t: z.literal("InternalError"),
    reason: z.string(),
  })
  ]);

export function adminRun(input: z.infer<typeof InputSchema>) {
  return callAction("admin_run", input, OutputSchema);
}
