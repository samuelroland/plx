// This a global Pinia store to store and change the state of everything related to a live session

import { defineStore } from "pinia";
import { CourseWithConfig, Exo } from "../ts/commands";
import {
  Event,
  ClientNum,
  ClientRole,
  ForwardedFile,
  Session,
  SessionStats,
  ExoCheckResult,
} from "../ts/shared";
import { LiveClient } from "../client";
import { useGlobalStore } from "./GlobalStore";
import { useTrainStore } from "./TrainStore";
import {
  convertLiveProtocolErrorToString,
  getRelativePathForExo,
  justNotify,
  NotifType,
} from "../util";

export interface Answer {
  client_num: ClientNum;
  last_timestamp: number;
  files: Map<string, ForwardedFile>;
  checks_status: Map<number, ExoCheckResult>;
}

export enum LiveSessionStep {
  EXOS_SELECTION,
  RUNNING,
}

export const useLiveStore = defineStore("live", {
  state: () => ({
    client: null as LiveClient | null,
    connectedTo: null as null | string, // to store the server id like "live.plx.rs:9120" or nothing. This also serves to know if we are connected
    clients_timeout_ids: [] as number[], // setTimeout id of callbacks to cancel
    client_num: -1 as number,
    course: null as CourseWithConfig | null,
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

    live_exos_ids: [] as string[], // the list of exos to train in a given session, chosen by a leader. Only for the leader.
    live_exos_map: new Map() as Map<string, Exo>, // a copy of the exo, indexed by path, to be able to show them during the training
    live_current_exo_index: 0 as number, // index inside live_exos_id of the current exo. Only for the leader.
    live_session_step: LiveSessionStep.EXOS_SELECTION as LiveSessionStep,
  }),
  getters: {
    // Only if the 3 fields are not null, the session is in progress
    sessionInProgress: (state) =>
      state.course && state.course.config && state.session,
    isConnected: (state) => state.connectedTo !== null,
  },
  actions: {
    connect_if_no_client() {
      const config = this.course?.config;
      if (!config) {
        justNotify(
          NotifType.Error,
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

    disconnect_if_existing_client() {
      if (this.client) {
        this.client.disconnect();
        this.client = null;
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
      if (this.course?.config?.group_id) {
        this.client?.send_msg({
          type: "GetSessions",
          content: { group_id: this.course.config?.group_id },
        });
      } else {
        justNotify(
          NotifType.Error,
          "This course has no valid live configuration, cannot connect to a live server.",
        );
      }
    },
    currentLiveExo() {
      const path = this.live_exos_ids[this.live_current_exo_index];
      return this.live_exos_map.get(path);
    },
    getCourseFolder() {
      return this.course?.course.folder + "/";
    },
    sendSwitchExoAction(path: string) {
      this.connect_if_no_client();
      const relative_path = getRelativePathForExo(path, this.getCourseFolder());
      this.client?.send_msg({
        type: "SwitchExo",
        content: { path: relative_path },
      });
    },
    changeLiveExoIndex(increment: number) {
      if (
        (increment < 0 && this.live_current_exo_index > 0) ||
        (increment > 0 &&
          this.live_current_exo_index < this.live_exos_ids.length - 1)
      ) {
        this.answers.clear();
        this.live_current_exo_index += increment;
        const newPath = this.currentLiveExo()?.folder;
        if (newPath) this.sendSwitchExoAction(newPath);
      }
    },
    startTraining() {
      this.live_current_exo_index = 0;
      this.live_session_step = LiveSessionStep.RUNNING;
      const firstExoPath = this.currentLiveExo()?.folder;
      if (firstExoPath) this.sendSwitchExoAction(firstExoPath);
    },
    sendCheckResult(check_result: ExoCheckResult) {
      this.connect_if_no_client();
      this.client?.send_msg({
        type: "SendResult",
        content: { check_result },
      });
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
        const relative_exo_path = event.content.path;
        if (live.role == ClientRole.Follower) {
          const absolute_path = live.getCourseFolder() + relative_exo_path;
          train.current_live_exo = train.findExo(absolute_path);
          if (!train.current_live_exo) {
            justNotify(
              NotifType.Error,
              "The leader has switched to the exo on folder " +
                relative_exo_path +
                " but absolute path " +
                absolute_path +
                " doesn't exist locally. Make sure the Git repository is up-to-date !",
            );
          } else {
            train.stopExo();
            train.startExo();
          }
        } // else do nothing, the leader already switched the exo via the state live.live_current_exo_index
      }
      break;

    case "ForwardFile": {
      let entry = live.answers.get(event.content.client_num);
      if (!entry)
        // init the entry if not existant
        entry = {
          client_num: event.content.client_num,
          checks_status: new Map(),
          files: new Map(),
          last_timestamp: 0,
        };

      entry.last_timestamp = event.content.file.time;
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
          last_timestamp: 0,
        };

      entry.last_timestamp = event.content.result.time;
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

      justNotify(
        NotifType.ServerError,
        convertLiveProtocolErrorToString(event.content),
        6000,
      );
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
