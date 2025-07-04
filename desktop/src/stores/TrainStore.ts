// This a global Pinia store to manage the state of everything related to training on code exos

import { defineStore } from "pinia";
import { commands, Project } from "../ts/commands";
import { useGlobalStore } from "./GlobalStore";

export const useTrainStore = defineStore("train", {
  state: () => ({
    course: null as Project | null,
  }),
  getters: {},
  actions: {
    async loadCourse(course_path: string) {
      const global = useGlobalStore();
      const result = await commands.getFullCourseDetails(course_path);
      if (result.status == "ok") {
        this.course = result.data;
        global.page = "course";
      } else {
        alert(result.error);
      }
    },
  },
});
