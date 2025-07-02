// This a global Pinia store to store and change the state of everything related to a live session

import { defineStore } from "pinia";
import { CourseInfo, DEFAULT_LIVE_PORT, Session } from "../ts/commands";
import {
  CheckStatus,
  ClientNum,
  ClientRole,
  ForwardedFile,
  LiveConfig,
  SessionStats,
} from "../ts/bindings";
import { LiveClient } from "../client";

export interface Answer {
  client_num: ClientNum;
  files: Map<string, ForwardedFile>;
  checks_status: CheckStatus[];
}

export const useLiveStore = defineStore("live", {
  state: () => ({
    client: null as LiveClient | null,
    course: null as CourseInfo | null,
    config: null as LiveConfig | null,
    role: ClientRole,
    session: null as Session | null,
    answers: new Map() as Map<number, Answer>,
    stats: null as SessionStats | null,

    waitingWsEvents: {} as { [key: string]: boolean },
  }),
  getters: {
    // Only if the 3 fields are not null, the session is in progress
    sessionInProgress: (state) => state.course && state.config && state.session,
  },
  actions: {
    connect() {
      this.client = LiveClient.connect(
        "127.0.0.1",
        DEFAULT_LIVE_PORT,
        "super id",
      );
    },
    start_session(name: string, group_id: string) {
      this.client?.send_msg({
        type: "StartSession",
        content: { name, group_id },
      });
    },
    join_session(name: string, group_id: string) {
      this.client?.send_msg({
        type: "JoinSession",
        content: { name, group_id },
      });
    },
    // Get sessions for the group_id in LiveConfig
    get_sessions() {
      if (this.config?.group_id) {
        this.client?.send_msg({
          type: "GetSessions",
          content: { group_id: this.config?.group_id },
        });
      } else {
        alert("No this.config?.group_id ");
      }
    },
  },
});
