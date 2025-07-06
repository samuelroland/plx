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

  static connect(
    domain: string,
    port: number,
    client_id: string,
    onEvent: (event: Event) => void,
  ): LiveClient {
    let querystring = new URLSearchParams();
    querystring.append(
      QUERYSTRING_LIVE_CLIENT_ID_FIELD,
      Math.random().toString(), // TODO: use a fixed and persisted client_id,
    );
    querystring.append(
      QUERYSTRING_LIVE_PROTOCOL_VERSION_FIELD,
      PROTOCOL_VERSION,
    );
    const socket = new WebSocket(
      "ws://" + domain + ":" + port + "/?" + querystring.toString(),
    );
    // TODO: manage connection failures !

    socket.onmessage = (msg: MessageEvent) => {
      try {
        let event = JSON.parse(msg.data) as Event;
        onEvent(event);
      } catch {
        console.warn("Got message with invalid JSON: ");
      }
    };
    return new LiveClient(socket);
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
  }
}
