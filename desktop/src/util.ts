import { useLiveStore } from "./stores/LiveStore";

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
