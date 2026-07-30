// Auto-generated from src/hooks/greet.rs

import { z } from "zod";
import { useForteHook } from "@forte/react";

const InputSchema = z.object({
    name: z.string(),
  });

const OutputSchema = z.object({
    greeting: z.string(),
    random: z.number(),
  });

export function useGreet(input: z.infer<typeof InputSchema>) {
  return useForteHook("greet", input, OutputSchema);
}
