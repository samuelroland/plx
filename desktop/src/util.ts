import { notify } from "notiwind";
import { useLiveStore } from "./stores/LiveStore";
import { LiveProtocolError } from "./ts/shared";

// Try to anonymize the given string, by removing identifiable path such as absolute path of the course folder
// TODO: could this cause XSS ??
export function anonymizeText(given: string) {
  const live = useLiveStore();
  const course_folder = live.course?.course.folder + "/";
  given = given.replaceAll(course_folder, "");
  return given;
}

// Transform an absolute path of an exo folder, into relative path that can be sent (in the SwitchExo action i.e.)
export function getRelativePathForExo(
  exo_absolute_path: string,
  course_folder: string,
) {
  if (exo_absolute_path.startsWith(course_folder)) {
    return exo_absolute_path.slice(course_folder.length);
  }
  return exo_absolute_path;
}

export enum NotifType {
  Success,
  Info,
  Error,
  ServerError,
  Debug,
}

// Wrapper of the "notify" function from notiwind to apply some defaults
// The duration is at minimum 3.5 seconds but is automatically increased if the text is longer
export function justNotify(type: NotifType, text: string, duration?: number) {
  const CHARS_TO_READ_PER_SECONDS = 30;
  const NOTIF_MIN_DURATION_MS = 3500;
  const estimatedTimeMs = (text.length / CHARS_TO_READ_PER_SECONDS) * 1000;
  let finalDuration =
    duration ??
    (estimatedTimeMs < NOTIF_MIN_DURATION_MS
      ? NOTIF_MIN_DURATION_MS
      : estimatedTimeMs);

  notify(
    {
      group: "notifs", // just a fixed value, the same as the group attributed given to NotificationGroup in NotifZone
      text,
      type,
    },
    finalDuration,
  );
}

export function convertLiveProtocolErrorToString(
  error: LiveProtocolError,
): string {
  switch (error.type) {
    case "FailedToStartSession":
      return `Failed to start a session: ${error.reason}`;
    case "FailedToJoinSession":
      return `Failed to join the session: ${error.reason}`;
    case "FailedToLeaveSession":
      return "No session joined, cannot leave the session.";
    case "FailedSendingWithoutSession":
      return "Failed to send file content or check result, because no session joined.";
    case "SessionNotFound":
      return "The session wasn't found.";
    case "CannotJoinOtherSession":
      return "You cannot join another session without having left your current session.";
    case "ForbiddenSessionStop":
      return "You are not the creator of this session, you cannot stop it";
    case "ActionOnlyForLeader":
      return `The action ${error.reason} is permitted to leaders of the session.`;
  }
}
