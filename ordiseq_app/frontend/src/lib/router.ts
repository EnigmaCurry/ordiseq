import { writable } from "svelte/store";

export type Page = "main" | "test" | "settings";

export const currentPage = writable<Page>("main");
