import { writable } from "svelte/store";

export type Page = "test" | "settings";

export const currentPage = writable<Page>("settings");
