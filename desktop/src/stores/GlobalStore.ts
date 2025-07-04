// This a global Pinia store to manage state about the application, that is not specific to one big feature

import { defineStore } from "pinia";

export type Page = "home" | "join" | "start" | "dashboard" | "course";

export const useGlobalStore = defineStore("global", {
  state: () => ({
    page: "home" as Page,
  }),
  getters: {},
  actions: {},
});
