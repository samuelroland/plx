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
    querystring.append(QUERYSTRING_LIVE_CLIENT_ID_FIELD, client_id);
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

  send_msg(action: Action) {
    console.log("Send action ", action);
    // If the socket is not yet fully connected, retry in 20ms
    if (this.socket?.readyState !== this.socket?.OPEN) {
      console.log("retry");
      setTimeout(() => {
        this.send_msg(action);
      }, 20);
      return;
    }
    this.socket?.send(JSON.stringify(action)); // fail here !
  }

  disconnect() {
    this.socket?.close();
  }
}
