// This a global Pinia store to manage the state of everything related to training on code exos

import { defineStore } from "pinia";
import { commands, Exo, Course, Skill } from "../ts/commands";
import { useGlobalStore } from "./GlobalStore";
import { Channel } from "@tauri-apps/api/core";
import { useLiveStore } from "./LiveStore";
import { complement, ParseError } from "../ts/complement";
import { ClientRole, ExoStatusReport } from "../ts/shared";
import { justNotify, NotifType } from "../util";

export const useTrainStore = defineStore("train", {
  state: () => ({
    // The complete course details -> course + skills details + exos details
    course: null as Course | null,
    errors: [] as ParseError[],

    exo_status: undefined as ExoStatusReport | undefined,

    in_live_session: false as boolean,

    current_live_exo: undefined as Exo | undefined,

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
      if (this.in_live_session) {
        return this.current_live_exo;
      } else {
        return this.course?.skills[this.selectedSkillIdx].exos[
          this.selectedExoIdx
        ];
      }
    },

    findExo(path: string) {
      for (const skill of this.course?.skills ?? []) {
        for (const exo of skill.exos) {
          if (exo.folder == path) {
            return exo;
          }
        }
      }
      return undefined;
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
      const channel = new Channel<ExoStatusReport>();
      channel.onmessage = this.onExoStatusChange;
      const result = await complement.loadFullCourseDetails(
        course_path,
        channel,
      );
      console.log(result);
      if (result.status == "ok") {
        this.course = result.data.course;
        this.errors = result.data.errors;
      } else {
        justNotify(NotifType.ServerError, result.error);
      }
    },

    onExoStatusChange(status: ExoStatusReport) {
      const live = useLiveStore();
      this.exo_status = status;
      if (status.compilation_success != this.exo_status.compilation_success) {
        // live.send_check(status)
      }
    },

    async startExo() {
      const exo = this.currentExo();
      if (!exo) return;
      await commands.sendUiActionToApp({
        type: "StartExo",
        content: { exo_folder: exo.folder },
      });
    },

    async stopExo() {
      this.exo_status = undefined;
      await commands.sendUiActionToApp({
        type: "StopExo",
      });
    },
  },
});
