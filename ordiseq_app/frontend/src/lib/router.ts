import { writable } from "svelte/store";

export type Page = "graphics" | "test" | "settings" | "clients";

export const currentPage = writable<Page>("clients");
