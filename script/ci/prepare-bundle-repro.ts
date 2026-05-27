const repoRoot = Deno.cwd();
const pathSeparator = Deno.build.os === "windows" ? "\\" : "/";

function joinPath(...parts: string[]): string {
  return parts
    .filter((part) => part.length > 0)
    .map((part, index) => {
      if (index === 0) {
        return part.replace(/[\\/]+$/g, "");
      }
      return part.replace(/^[\\/]+|[\\/]+$/g, "");
    })
    .join(pathSeparator);
}

function dirnamePath(path: string): string {
  const slash = path.lastIndexOf("/");
  const backslash = path.lastIndexOf("\\");
  const index = Math.max(slash, backslash);
  return index === -1 ? "." : path.slice(0, index);
}

const normalizeScript = joinPath(
  repoRoot,
  "script",
  "ci",
  "normalize-bundle-inputs.ts",
);

async function exists(path: string): Promise<boolean> {
  try {
    await Deno.lstat(path);
    return true;
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) {
      return false;
    }
    throw error;
  }
}

async function runCommand(
  command: string,
  args: string[],
  options: Deno.CommandOptions = {},
): Promise<void> {
  const child = new Deno.Command(command, {
    ...options,
    args,
    stdout: "inherit",
    stderr: "inherit",
  });
  const status = await child.spawn().status;

  if (!status.success) {
    const code = status.code ?? 1;
    throw new Error(
      `${command} ${args.join(" ")} failed with exit code ${code}`,
    );
  }
}

async function normalizeBundleInputs(): Promise<void> {
  await runCommand(Deno.execPath(), ["run", "-A", normalizeScript], {
    cwd: repoRoot,
  });
}

const windowsMakensisWrapperSource = String.raw`
use std::{
    env,
    ffi::OsString,
    path::PathBuf,
    process::{exit, Command},
};

fn exit_from_status(status: std::process::ExitStatus) -> ! {
    exit(status.code().unwrap_or(1));
}

fn main() {
    let current_exe = env::current_exe().expect("current executable path");
    let real_makensis = current_exe.with_file_name("makensis-real.exe");

    if env::var_os("SOURCE_DATE_EPOCH").is_some() {
        let repo_root = env::var_os("SONAR_REPO_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| env::current_dir().expect("current directory"));
        let normalize_script = repo_root
            .join("script")
            .join("ci")
            .join("normalize-bundle-inputs.ts");

        eprintln!(
            "sonar reproducible makensis wrapper: normalizing bundle inputs before NSIS"
        );

        let status = Command::new("deno")
            .arg("run")
            .arg("-A")
            .arg(normalize_script)
            .status()
            .expect("failed to run Deno bundle input normalization");

        if !status.success() {
            exit_from_status(status);
        }
    }

    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let status = Command::new(real_makensis)
        .args(args)
        .status()
        .expect("failed to run real makensis executable");

    exit_from_status(status);
}
`;

async function installWindowsMakensisWrapper(): Promise<void> {
  if (Deno.build.os !== "windows") {
    return;
  }

  if (!Deno.env.get("SOURCE_DATE_EPOCH")) {
    return;
  }

  const localAppData = Deno.env.get("LOCALAPPDATA");
  const candidates = [
    localAppData ? joinPath(localAppData, "tauri", "NSIS", "makensis.exe") : "",
    joinPath(repoRoot, "src-tauri", "target", ".tauri", "NSIS", "makensis.exe"),
  ].filter(Boolean);

  for (const makensisPath of candidates) {
    if (!(await exists(makensisPath))) {
      continue;
    }

    const wrapperDir = dirnamePath(makensisPath);
    const realMakensisPath = joinPath(wrapperDir, "makensis-real.exe");

    if (!(await exists(realMakensisPath))) {
      await Deno.rename(makensisPath, realMakensisPath);
    }

    const sourcePath = joinPath(wrapperDir, "makensis-repro-wrapper.rs");
    await Deno.writeTextFile(sourcePath, windowsMakensisWrapperSource);
    await runCommand("rustc", [
      sourcePath,
      "-C",
      "opt-level=2",
      "-o",
      makensisPath,
    ]);

    console.log(`Installed reproducible makensis wrapper at ${makensisPath}.`);
    return;
  }

  console.log(
    "No Tauri makensis.exe cache found yet; PATH wrapper remains the fallback.",
  );
}

await normalizeBundleInputs();
await installWindowsMakensisWrapper();
