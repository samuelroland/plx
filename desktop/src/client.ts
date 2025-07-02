import {
  PROTOCOL_VERSION,
  QUERYSTRING_LIVE_CLIENT_ID_FIELD,
  QUERYSTRING_LIVE_PROTOCOL_VERSION_FIELD,
} from "./ts/commands";
import { Event, Action } from "./ts/bindings.ts";
import { useLiveStore } from "./stores/LiveStore.ts";

QUERYSTRING_LIVE_CLIENT_ID_FIELD;
export class LiveClient {
  socket: WebSocket | undefined;
  private constructor(socket: WebSocket) {
    this.socket = socket;
  }

  static connect(domain: string, port: number, client_id: string): LiveClient {
    let querystring = new URLSearchParams();
    querystring.append(QUERYSTRING_LIVE_CLIENT_ID_FIELD, client_id);
    querystring.append(
      QUERYSTRING_LIVE_PROTOCOL_VERSION_FIELD,
      PROTOCOL_VERSION,
    );
    const socket = new WebSocket(
      "ws://" + domain + ":" + port + "/?" + querystring.toString(),
    );

    socket.onmessage = (event: MessageEvent) => {
      onMessage(event);
    };
    return new LiveClient(socket);
  }

  send_msg(action: Action) {
    this.socket?.send(JSON.stringify(action));
  }

  disconnect() {
    this.socket?.close();
  }
}

function onMessage(message: MessageEvent): any {
  try {
    let event = JSON.parse(message.data) as Event;
    onEvent(event);
  } catch {}
}

function onEvent(event: Event) {
  const live = useLiveStore();
  console.log("Got event", event);
  switch (event.type) {
    case "SessionStarted":
      break;
    case "SessionStopped":
      break;
    case "SessionJoined":
      break;
    case "SessionsList":
      break;
    case "ServerStopped":
      break;
    case "ExoSwitched":
      break;

    case "ForwardFile": {
      let entry = live.answers.get(event.content.client_num);
      if (!entry)
        entry = {
          client_num: event.content.client_num,
          checks_status: [],
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
          checks_status: [],
          files: new Map(),
        };
      if (!entry)
        entry = {
          client_num: event.content.client_num,
          checks_status: [],
          files: new Map(),
        };
      entry.checks_status.push(event.content.result.check_result.state);
      live.answers.set(event.content.client_num, entry);
      break;
    case "Stats":
      live.stats = event.content;
  }
}
