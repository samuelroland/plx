// Hand written file to complement Tauri commands signatures that Tauri-specta cannot generate manually
import {
  invoke as TAURI_INVOKE,
  Channel as TAURI_CHANNEL,
} from "@tauri-apps/api/core";

export const complement = {
  async setup_answers_streaming(c: TAURI_CHANNEL): Promise<void> {
    return await TAURI_INVOKE("get_local_courses");
  },
};
