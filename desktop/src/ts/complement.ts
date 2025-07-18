// Hand written file to complement Tauri commands signatures that Tauri-specta cannot generate manually
// Some types of the paramaters are exported with typeshare
import {
  invoke as TAURI_INVOKE,
  Channel as TAURI_CHANNEL,
} from "@tauri-apps/api/core";
import { Result } from "./commands";
import { CourseWithErrors, ExoStatusReport } from "./shared";

export const complement = {
  async loadFullCourseDetails(
    coursePath: string,
    exoStatusUiChannel: TAURI_CHANNEL<ExoStatusReport>,
  ): Promise<Result<CourseWithErrors, string>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE("load_full_course_details", {
          coursePath,
          exoStatusUiChannel,
        }),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      else return { status: "error", error: e as any };
    }
  },
};
