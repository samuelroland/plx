// This a global Pinia store to manage the state of everything related to training on code exos

import { defineStore } from "pinia";
import { commands, Exo, Course, Skill } from "../ts/commands";
import { useGlobalStore } from "./GlobalStore";
import { Channel } from "@tauri-apps/api/core";
import { useLiveStore } from "./LiveStore";
import { complement, ParseError } from "../ts/complement";
import { ClientRole, ExoCheckResult, ExoStatusReport } from "../ts/shared";
import { anonymizeAndSimplifyText, justNotify, NotifType } from "../util";

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
      if (result.status == "ok") {
        this.course = result.data.course;
        this.errors = result.data.errors;
      } else {
        justNotify(NotifType.ServerError, result.error);
      }
    },

    // When the backend send us a change of the exo status, we need to update the UI
    // and if we are in a live session, we also need to send the updated files
    // and results of the check
    onExoStatusChange(status: ExoStatusReport) {
      this.exo_status = status;
      // TODO: only send if it has changed !

      // Nothing more to do if we are not in a live session
      // we don't want to send the result to the server !!
      if (!this.in_live_session) return;

      const live = useLiveStore();
      status.edited_files_content.forEach((file) => {
        live.sendFile(file);
      });

      if (status.compilation_running) return; // nothing to send for now

      // For each check, we convert the type of the result to the type that is valid for ExoCheckResult
      // and we send this result to the server
      status.check_results.forEach((result, idx) => {
        // If it doesn't build, we send a BuildFailed state for each check
        // TODO: refactor this non performant strategy with a custom protocol message to send build errors
        if (!status.compilation_success) {
          const finalResult: ExoCheckResult = {
            index: idx,
            state: {
              type: "BuildFailed",
              content: anonymizeAndSimplifyText(status.compilation_output),
            },
          };
          live.sendCheckResult(finalResult);
        } else {
          // convert to the correct "type", extract the content and send it
          const type = result.state.status.type;
          let newType:
            | "Passed"
            | "CheckFailed"
            | "BuildFailed"
            | "RunFailed"
            | undefined;
          let content: string = "";
          switch (type) {
            case "Passed":
              newType = "Passed";
              break;
            case "Failed":
              newType = "CheckFailed";
              content = result.state.status.content.given;
              break;
            case "RunFail":
              newType = "RunFailed";
              content = result.state.status.content;
              break;
            default:
              return; // continue with next check result
          }

          if (newType != undefined) {
            let finalResult: ExoCheckResult;
            if (newType == "Passed") {
              finalResult = {
                index: idx,
                state: { type: newType },
              };
            } else {
              finalResult = {
                index: idx,
                state: { type: newType, content },
              };
            }
            live.sendCheckResult(finalResult);
          }
        }
      });
      // TODO: only send checks results that have changed !
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
