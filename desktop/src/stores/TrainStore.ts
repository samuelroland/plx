// This a global Pinia store to manage the state of everything related to training on code exos

import { defineStore } from "pinia";
import { commands, Exo, Project, Skill } from "../ts/commands";
import { useGlobalStore } from "./GlobalStore";

export const useTrainStore = defineStore("train", {
  state: () => ({
    // The complete course details -> course + skills details + exos details
    course: null as Project | null,

    // Skills and exo selection, used in Course and Train
    selectedSkillIdx: 0,
    selectedExoIdx: 0,
    exosSelection: false, // skills selection by default, exos selection with right arrow or l key
  }),
  getters: {},
  actions: {
    currentSkill(): Skill | undefined {
      return this.course?.skills[this.selectedSkillIdx];
    },
    currentExo(): Exo | undefined {
      return this.course?.skills[this.selectedSkillIdx].exos[
        this.selectedExoIdx
      ];
    },

    switchExo(increment: number) {
      const length = this.currentSkill()?.exos.length;
      if (length) {
        let newIndex = this.selectedExoIdx;
        newIndex += increment;
        if (newIndex >= length) {
          newIndex = length - 1;
        } else if (newIndex < 0) {
          newIndex = 0;
        }
        this.selectedExoIdx = newIndex;
      }
    },

    switchSkill(increment: number) {
      if (this.course?.skills.length) {
        let newIndex = this.selectedSkillIdx;
        newIndex += increment;
        if (newIndex >= this.course?.skills.length) {
          newIndex = this.course?.skills.length - 1;
        } else if (newIndex < 0) {
          newIndex = 0;
        }
        this.selectedSkillIdx = newIndex;
      }
    },

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
