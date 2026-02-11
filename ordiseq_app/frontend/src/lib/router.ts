import { writable } from "svelte/store";

export type Page = "graphics" | "settings" | "clients";

export const currentPage = writable<Page>("clients");
