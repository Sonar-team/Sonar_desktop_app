#!/usr/bin/env -S deno run -A

/**
 * Centralizes the reproducibility-oriented environment used by SONAR release builds.
 *
 * Context:
 * - The project already had a standalone reproducibility check script that set
 *   `SOURCE_DATE_EPOCH` and `RUSTFLAGS=--remap-path-prefix=...` before running
 *   Tauri builds.
 * - That was useful for experiments, but it did not guarantee that the real
 *   release path used the same inputs.
 * - This helper closes that gap by giving CI release jobs, local release
 *   commands, and reproducibility checks one shared way to inject the same
 *   environment variables into the build.
 *
 * What this script does:
 * - resolves a stable `SOURCE_DATE_EPOCH`
 * - adds Rust path remapping so local absolute paths do not leak into outputs
 * - exposes the environment in a GitHub Actions-friendly format
 * - can also execute an arbitrary build command with that environment applied
 *
 * Expected usage:
 * - `github-env`: used by `.github/workflows/publish.yml` to append variables to
 *   `$GITHUB_ENV` before `tauri-action` runs
 * - `run ...`: used by local release commands and by `security/repro-check.sh`
 *   to execute a build with the same reproducibility settings
 *
 * The goal is not to solve every source of non-determinism by itself. It
 * specifically handles the release-build flags that must be applied consistently
 * at the build entrypoint so repeated builds of the same revision have a better
 * chance of producing identical artifacts.
 */
const repoRoot = Deno.cwd();

// Preserve any existing flags while ensuring the reproducibility remap is present exactly once.
function appendFlag(existing: string | undefined, flag: string): string {
  const parts = (existing ?? "").split(/\s+/).filter(Boolean);
  if (parts.includes(flag)) {
    return parts.join(" ");
  }
  parts.push(flag);
  return parts.join(" ");
}

async function resolveSourceDateEpoch(): Promise<string> {
  // Allow callers to pin the timestamp explicitly, which is useful for CI and repro checks.
  const existing = Deno.env.get("SOURCE_DATE_EPOCH");
  if (existing) {
    return existing;
  }

  // Fall back to the last commit timestamp so rebuilds of the same revision share one epoch.
  const command = new Deno.Command("git", {
    args: ["log", "-1", "--format=%ct", "HEAD"],
    cwd: repoRoot,
    stdout: "piped",
    stderr: "null",
  });
  const { code, stdout } = await command.output();
  if (code !== 0) {
    throw new Error("Unable to derive SOURCE_DATE_EPOCH from git history");
  }

  const epoch = new TextDecoder().decode(stdout).trim();
  if (!/^\d+$/.test(epoch)) {
    throw new Error(`Invalid SOURCE_DATE_EPOCH derived from git: ${epoch}`);
  }
  return epoch;
}

async function buildEnv(): Promise<Record<string, string>> {
  const sourceDateEpoch = await resolveSourceDateEpoch();
  // Remap the local checkout path to a stable virtual path to avoid embedding machine-specific paths.
  const remapFlag = `--remap-path-prefix=${repoRoot}=/workspace`;
  const rustflags = appendFlag(Deno.env.get("RUSTFLAGS"), remapFlag);

  return {
    SOURCE_DATE_EPOCH: sourceDateEpoch,
    RUSTFLAGS: rustflags,
  };
}

function printGithubEnv(envVars: Record<string, string>): void {
  // Use the multiline GitHub Actions env-file format so values survive shell parsing unchanged.
  for (const [key, value] of Object.entries(envVars)) {
    console.log(`${key}<<__SONAR_REPRO_ENV__`);
    console.log(value);
    console.log("__SONAR_REPRO_ENV__");
  }
}

async function runWithEnv(commandArgs: string[]): Promise<number> {
  if (commandArgs.length === 0) {
    throw new Error("Missing command to run");
  }

  const envVars = await buildEnv();
  // Spawn the target command with the reproducibility env merged into the current process env.
  const command = new Deno.Command(commandArgs[0], {
    args: commandArgs.slice(1),
    cwd: repoRoot,
    env: {
      ...Deno.env.toObject(),
      ...envVars,
    },
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit",
  });

  const { code } = await command.spawn().status;
  return code;
}

async function main(): Promise<void> {
  const [mode, ...args] = Deno.args;

  switch (mode) {
    case "github-env": {
      // Emit shell-safe env assignments for GitHub Actions to append into $GITHUB_ENV.
      printGithubEnv(await buildEnv());
      return;
    }
    case "run": {
      // Run an arbitrary build command with the same reproducibility settings applied locally.
      const code = await runWithEnv(args);
      Deno.exit(code);
      return;
    }
    default:
      throw new Error("Usage: deno run -A security/repro-env.ts <github-env|run ...>");
  }
}

if (import.meta.main) {
  await main();
}
