// This a global Pinia store to store and change the state of everything related to a live session

import { defineStore } from "pinia";
import { commands, CourseInfo, DEFAULT_LIVE_PORT } from "../ts/commands";
import {
  Event,
  Action,
  CheckStatus,
  ClientNum,
  ClientRole,
  ForwardedFile,
  Session,
  SessionStats,
  ExoCheckResult,
} from "../ts/bindings";
import { LiveClient } from "../client";
import { useGlobalStore } from "./GlobalStore";
import { useTrainStore } from "./TrainStore";

export interface Answer {
  client_num: ClientNum;
  files: Map<string, ForwardedFile>;
  checks_status: Map<number, ExoCheckResult>;
}

export const useLiveStore = defineStore("live", {
  state: () => ({
    client: null as LiveClient | null,
    client_num: -1 as number,
    course: null as CourseInfo | null,
    // Sessions available for the selected course, kept empty when no course
    available_sessions: [] as Session[],
    role: ClientRole.Follower,
    session: null as Session | null,
    answers: new Map() as Map<number, Answer>,
    stats: null as SessionStats | null,
    // Some temporary states, usualy in waiting a websocket Event back after an Action
    tmp: {
      joining_session: null as Session | null, // is not null between JoinSession and SessionJoined/Error
      starting_session: null as Session | null, // is not null between StartSession and SessionJoined/Error
    },
  }),
  getters: {
    // Only if the 3 fields are not null, the session is in progress
    sessionInProgress: (state) =>
      state.course && state.course.config && state.session,
  },
  actions: {
    connect_if_no_client() {
      const config = this.course?.config;
      if (!config) {
        alert(
          "No live.toml configuration found in course repository, cannot connect to a live server !",
        );
        return;
      }
      if (!this.client) {
        this.client = LiveClient.connect(
          config.domain,
          config.port,
          "super id",
          onEvent,
        );
      }
    },
    start_session(name: string) {
      this.connect_if_no_client();
      if (this.course?.config?.group_id) {
        let session = { name, group_id: this.course?.config?.group_id };
        this.client?.send_msg({
          type: "StartSession",
          content: session,
        });
        this.tmp.starting_session = session;
      }
    },
    join_session(name: string) {
      this.connect_if_no_client();
      if (this.course?.config?.group_id) {
        let session = { name, group_id: this.course?.config?.group_id };
        this.client?.send_msg({
          type: "JoinSession",
          content: session,
        });
        this.tmp.joining_session = session;
      }
    },
    leave_session() {
      this.connect_if_no_client();
      this.client?.send_msg({
        type: "LeaveSession",
      });
    },
    // Get sessions for the group_id in LiveConfig
    get_sessions() {
      this.available_sessions = [];
      this.connect_if_no_client();
      console.log("okay");
      if (this.course?.config?.group_id) {
        this.client?.send_msg({
          type: "GetSessions",
          content: { group_id: this.course.config?.group_id },
        });
      } else {
        alert(
          "This course has no valid live configuration, cannot connect to a live server.",
        );
      }
    },
  },
});

// Define the logic to react on an Event received from the server via LiveClient
function onEvent(event: Event) {
  const live = useLiveStore();
  const train = useTrainStore();
  console.log("Got event", event);
  switch (event.type) {
    case "SessionStopped":
      break;
    case "SessionJoined":
      live.client_num = event.content;
      if (live.tmp.joining_session != null) {
        live.session = live.tmp.joining_session;
        live.tmp.joining_session = null;
        live.role = ClientRole.Follower;
      }
      if (live.tmp.starting_session != null) {
        live.session = live.tmp.starting_session;
        live.tmp.starting_session = null;
        live.role = ClientRole.Leader;
      }
      break;
    case "SessionsList":
      live.available_sessions = event.content;
      break;
    case "ServerStopped":
      break;
    case "ExoSwitched":
      if (train.in_live_session) {
        train.current_live_exo = train.findExo(event.content.path);
        if (!train.current_live_exo) {
          alert("exo not found " + event.content.path);
        }
      }
      break;

    case "ForwardFile": {
      let entry = live.answers.get(event.content.client_num);
      if (!entry)
        entry = {
          client_num: event.content.client_num,
          checks_status: new Map(),
          files: new Map(),
        };
      entry.files.set(event.content.file.path, event.content.file);
      live.answers.set(event.content.client_num, entry);
      break;
    }
    case "ForwardResult":
      let entry = live.answers.get(event.content.client_num);
      if (!entry)
        entry = {
          client_num: event.content.client_num,
          checks_status: new Map(),
          files: new Map(),
        };
      if (!entry)
        entry = {
          client_num: event.content.client_num,
          checks_status: new Map(),
          files: new Map(),
        };
      entry.checks_status.set(
        event.content.result.check_result.index,
        event.content.result.check_result,
      );
      live.answers.set(event.content.client_num, entry);
      break;
    case "Stats":
      live.stats = event.content;
      break;
    case "Error":
      console.error(event.content);
      alert(event.content);
      break;
  }

  const global = useGlobalStore();
  // Handle page switch
  if (live.sessionInProgress) {
    if (live.role == ClientRole.Follower) {
      train.in_live_session = true;
      global.page = "train";
    } else {
      global.page = "dashboard";
    }
  }

  console.log("liveStore", live);
}
