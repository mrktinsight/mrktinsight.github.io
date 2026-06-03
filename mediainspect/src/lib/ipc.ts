import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { InspectReport, ToolStatus } from "./types";

export async function pickFile(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [
      {
        name: "Media files",
        extensions: [
          "mp4", "mov", "m4v", "m4a", "mkv", "webm", "ts", "m2ts",
          "mxf", "wav", "flac", "aac", "mp3", "ogg", "opus", "aif",
          "aiff", "caf",
        ],
      },
      { name: "All files", extensions: ["*"] },
    ],
  });
  if (typeof selected === "string") return selected;
  return null;
}

export async function inspect(path: string): Promise<InspectReport> {
  return invoke<InspectReport>("inspect", { path });
}

export async function toolStatus(): Promise<ToolStatus> {
  return invoke<ToolStatus>("tool_status");
}
