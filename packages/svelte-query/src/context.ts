import { AlphaClient, ProceduresDef } from "@rspc/client";
import { getContext, setContext } from "svelte";
import { Context } from "@rspc/query-core";

const _contextKey = "$$_rspcClient";

/** Retrieves a Client from Svelte's context */
export const getRspcClientContext = <
  P extends ProceduresDef
>(): Context<P> | null => {
  const ctx = getContext(_contextKey) ?? null;
  return ctx as Context<P> | null;
};

/** Sets a Client on Svelte's context */
export const setRspcClientContext = (client: AlphaClient<any>) =>
  setContext(_contextKey, client);
