// Auto-generated from src/actions/doc_query.rs

import { z } from "zod";
import { callAction } from "@forte/react";

const InputStatementSchema = z.object({
    sql: z.string(),
    args: z.array(z.json()),
  });

const CellValueSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Null"),
  }),
    z.object({
    t: z.literal("Integer"),
    value: z.number(),
  }),
    z.object({
    t: z.literal("Float"),
    value: z.number(),
  }),
    z.object({
    t: z.literal("Text"),
    value: z.string(),
  }),
    z.object({
    t: z.literal("Blob"),
    base64: z.string(),
  })
  ]);

const StatementResultSchema = z.object({
    columnNames: z.array(z.string()),
    rows: z.array(z.array(CellValueSchema)),
    rowsTruncated: z.boolean(),
    affectedRowCount: z.number(),
    rowsRead: z.number(),
    rowsWritten: z.number(),
    queryDurationMs: z.number(),
  });

const InputSchema = z.object({
    projectId: z.string(),
    statements: z.array(InputStatementSchema),
  });

const OutputSchema = z.discriminatedUnion("t", [
    z.object({
    t: z.literal("Committed"),
    statementResults: z.array(StatementResultSchema),
  }),
    z.object({
    t: z.literal("RolledBack"),
    failedStatementIndex: z.number(),
    errorMessage: z.string(),
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
    t: z.literal("TooManyStatements"),
    max: z.number(),
  }),
    z.object({
    t: z.literal("InvalidArgument"),
    statementIndex: z.number(),
    argumentIndex: z.number(),
    reason: z.string(),
  }),
    z.object({
    t: z.literal("InternalError"),
    reason: z.string(),
  })
  ]);

export function docQuery(input: z.infer<typeof InputSchema>) {
  return callAction("doc_query", input, OutputSchema);
}
