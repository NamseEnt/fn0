// Auto-generated from src/actions/project_log_attribute_values.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const AttributeEqualsInputSchema = z.object({
    key: z.string(),
    value: z.string(),
  });

const InputSchema = z.object({
    projectId: z.string(),
    key: z.string(),
    start: z.string(),
    end: z.string().optional(),
    attributes: z.array(AttributeEqualsInputSchema),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Ok"),
    values: z.array(z.string()),
  }),
    z.object({
    t: z.literal("NotLoggedIn"),
  }),
    z.object({
    t: z.literal("NotFound"),
  }),
    z.object({
    t: z.literal("Error"),
    message: z.string(),
  }),
    z.object({
    t: z.literal("InternalError"),
  })
  ]);

export function projectLogAttributeValues(input: z.infer<typeof InputSchema>) {
  return callAction("project_log_attribute_values", input, OutputSchema);
}
