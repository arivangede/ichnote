// Typed wrappers around Tauri commands. Every frontend call into Rust goes through here.
import { invoke } from "@tauri-apps/api/core";

export function getAppVersion(): Promise<string> {
  return invoke<string>("app_version");
}
