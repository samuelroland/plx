import {
  PROTOCOL_VERSION,
  QUERYSTRING_LIVE_CLIENT_ID_FIELD,
  QUERYSTRING_LIVE_PROTOCOL_VERSION_FIELD,
} from "./ts/commands";
import { Event, Action } from "./ts/shared.ts";
import { useLiveStore } from "./stores/LiveStore.ts";
import { justNotify, NotifType } from "./util.ts";

// The maximum time the WebSocket connection can wait before a connection to the server
// We consider the server to be down otherwise.
const WEBSOCKET_CONNECTION_TIMEOUT_MS = 4000;
// The time before the "trying to connect" message appears
const WEBSOCKET_TRYING_MESSAGE_DELAY = 1000;

QUERYSTRING_LIVE_CLIENT_ID_FIELD;
export class LiveClient {
  socket: WebSocket | undefined;
  private constructor(socket: WebSocket) {
    this.socket = socket;
  }

  static connect(
    domain: string,
    port: number,
    client_id: string,
    onEvent: (event: Event) => void,
  ): LiveClient {
    const live = useLiveStore();
    live.clients_timeout_ids.map((id) => clearInterval(id));
    let querystring = new URLSearchParams();
    querystring.append(
      QUERYSTRING_LIVE_CLIENT_ID_FIELD,
      Math.random().toString(), // TODO: use a fixed and persisted client_id,
    );
    querystring.append(
      QUERYSTRING_LIVE_PROTOCOL_VERSION_FIELD,
      PROTOCOL_VERSION,
    );

    const serverVisualId = domain + ":" + port;
    const socket = new WebSocket(
      "ws://" + domain + ":" + port + "/?" + querystring.toString(),
    );

    const client = new LiveClient(socket);
    socket.onopen = () => {
      live.connectedTo = serverVisualId;
      live.clients_timeout_ids.map((id) => clearInterval(id));
      justNotify(
        NotifType.Success,
        "Connected to live server on " + serverVisualId,
      );
    };
    socket.onclose = () => {
      live.connectedTo = null;
    };

    const timeoutId1 = setTimeout(() => {
      justNotify(
        NotifType.Info,
        "Trying to connect to server " + serverVisualId + "...",
        WEBSOCKET_CONNECTION_TIMEOUT_MS - WEBSOCKET_TRYING_MESSAGE_DELAY + 200,
      );
    }, WEBSOCKET_TRYING_MESSAGE_DELAY);

    const timeoutId2 = setTimeout(() => {
      justNotify(
        NotifType.ServerError,
        "The connection to the live server " +
          serverVisualId +
          "\nhas failed to open under " +
          WEBSOCKET_CONNECTION_TIMEOUT_MS +
          "ms.\nEither you don't have internet access or the server is down...",
      );
    }, WEBSOCKET_CONNECTION_TIMEOUT_MS);
    live.clients_timeout_ids.push(timeoutId1);
    live.clients_timeout_ids.push(timeoutId2);
    socket.onmessage = (msg: MessageEvent) => {
      try {
        let event = JSON.parse(msg.data) as Event;
        onEvent(event);
      } catch {
        console.warn("Got message with invalid JSON: ");
      }
    };
    return client;
  }

  async send_msg(action: Action) {
    const MAX_RETRIES = 5;
    const RETRY_DELAY_MS = 100;

    console.log("Send action ", action);

    for (let try_count = 0; try_count < MAX_RETRIES; try_count++) {
      if (this.socket?.readyState === this.socket?.OPEN) {
        this.socket?.send(JSON.stringify(action));
        return;
      }
      console.log("WebSocket is not opened yet, waiting before trying again");
      await new Promise((resolve) => setTimeout(resolve, RETRY_DELAY_MS));
    }
  }

  disconnect() {
    this.socket?.close();
    const live = useLiveStore();
    live.connectedTo = null;
  }
}
