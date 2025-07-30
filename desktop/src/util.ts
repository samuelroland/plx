import { useLiveStore } from "./stores/LiveStore";

// Try to anonymize the given string, by removing identifiable path such as absolute path of the course folder
// TODO: could this cause XSS ??
export function anonymizeText(given: string) {
  const live = useLiveStore();
  const course_folder = live.course?.course.folder + "/";
  given = given.replaceAll(course_folder, "");
  return given;
}
