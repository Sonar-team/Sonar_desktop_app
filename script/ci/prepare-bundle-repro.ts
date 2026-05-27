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

function readBuildVersionsEnv(): Record<string, string> {
  const path = joinPath(repoRoot, "config", "build-versions.env");
  const values: Record<string, string> = {};

  try {
    const content = Deno.readTextFileSync(path);
    for (const line of content.split(/\r?\n/)) {
      const match = line.match(/^([A-Z0-9_]+)=(.*)$/);
      if (!match) {
        continue;
      }

      const [, key, rawValue] = match;
      values[key] = rawValue.replace(/^"|"$/g, "");
    }
  } catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) {
      throw error;
    }
  }

  return values;
}

const buildVersions = readBuildVersionsEnv();

function buildVersion(name: string): string {
  const value = Deno.env.get(name) ?? buildVersions[name];
  if (!value) {
    throw new Error(`Missing build version variable: ${name}`);
  }
  return value;
}

const tauriNsisVersion = buildVersion("NSIS_VERSION");
const tauriNsisZipUrl =
  `https://github.com/tauri-apps/binary-releases/releases/download/nsis-${tauriNsisVersion}/nsis-${tauriNsisVersion}.zip`;
const tauriNsisZipSha256 = buildVersion("NSIS_ZIP_SHA256");
const tauriNsisUtilsVersion = buildVersion("NSIS_TAURI_UTILS_VERSION");
const tauriNsisUtilsUrl =
  `https://github.com/tauri-apps/nsis-tauri-utils/releases/download/nsis_tauri_utils-v${tauriNsisUtilsVersion}/nsis_tauri_utils.dll`;
const tauriNsisUtilsSha1 = buildVersion("NSIS_TAURI_UTILS_SHA1");

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

async function removeIfExists(path: string): Promise<void> {
  try {
    await Deno.remove(path, { recursive: true });
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) {
      return;
    }
    throw error;
  }
}

async function normalizeBundleInputs(): Promise<void> {
  await runCommand(Deno.execPath(), ["run", "-A", normalizeScript], {
    cwd: repoRoot,
  });
}

const windowsMakensisWrapperSource = String.raw`
use std::{
    convert::TryFrom,
    env,
    ffi::OsString,
    fs,
    io,
    path::PathBuf,
    process::{exit, Command},
};

fn exit_from_status(status: std::process::ExitStatus) -> ! {
    exit(status.code().unwrap_or(1));
}

fn read_u16_le(data: &[u8], offset: usize) -> Option<u16> {
    let bytes = data.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_u32_le(data: &[u8], offset: usize) -> Option<u32> {
    let bytes = data.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn write_u32_le(data: &mut [u8], offset: usize, value: u32) -> Option<()> {
    let bytes = data.get_mut(offset..offset.checked_add(4)?)?;
    bytes.copy_from_slice(&value.to_le_bytes());
    Some(())
}

fn normalize_pe(path: PathBuf) -> io::Result<()> {
    let source_date_epoch = env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0);
    let mut data = fs::read(&path)?;

    if data.len() < 0x40 || data.get(0..2) != Some(b"MZ") {
        return Ok(());
    }

    let pe_offset = match read_u32_le(&data, 0x3c)
        .and_then(|value| usize::try_from(value).ok())
    {
        Some(value) => value,
        None => return Ok(()),
    };
    if data.get(pe_offset..pe_offset.saturating_add(4)) != Some(b"PE\0\0") {
        return Ok(());
    }

    let coff_header_offset = pe_offset.saturating_add(4);
    let optional_header_offset = coff_header_offset.saturating_add(20);
    let optional_header_size = read_u16_le(&data, coff_header_offset.saturating_add(16))
        .map(usize::from)
        .unwrap_or(0);

    let _ = write_u32_le(
        &mut data,
        coff_header_offset.saturating_add(4),
        source_date_epoch,
    );
    if optional_header_size >= 68 {
        let _ = write_u32_le(&mut data, optional_header_offset.saturating_add(64), 0);
    }

    fs::write(path, data)
}

fn escape_nsis_define(value: &str) -> String {
    value.replace('$', "$$")
}

fn patch_installer_script(script_path: PathBuf, wrapper_path: PathBuf) -> io::Result<()> {
    let content = fs::read_to_string(&script_path)?;
    if !content.contains("!define UNINSTALLERSIGNCOMMAND \"\"") {
        return Ok(());
    }

    let wrapper = wrapper_path.to_string_lossy().replace('\\', "/");
    let command = format!(
        "!define UNINSTALLERSIGNCOMMAND '\"{}\" --sonar-normalize-pe \"%1\"'",
        escape_nsis_define(&wrapper)
    );
    let patched = content.replacen("!define UNINSTALLERSIGNCOMMAND \"\"", &command, 1);

    fs::write(script_path, patched)
}

fn main() {
    let current_exe = env::current_exe().expect("current executable path");
    let args: Vec<OsString> = env::args_os().skip(1).collect();

    if args.first().and_then(|arg| arg.to_str()) == Some("--sonar-normalize-pe") {
        let Some(path) = args.get(1) else {
            eprintln!("sonar reproducible makensis wrapper: missing PE path to normalize");
            exit(2);
        };
        normalize_pe(PathBuf::from(path)).expect("failed to normalize PE file");
        return;
    }

    let current_dir = current_exe.parent().expect("current executable directory");
    let real_makensis = current_dir
        .with_file_name("NSIS-real")
        .join("makensis.exe");

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

        if let Some(script_path) = args
            .iter()
            .map(|arg| PathBuf::from(arg))
            .find(|path| {
                path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("nsi"))
            })
        {
            patch_installer_script(script_path, current_exe)
                .expect("failed to patch NSIS installer script");
        }
    }

    let status = Command::new(real_makensis)
        .args(args)
        .status()
        .expect("failed to run real makensis executable");

    exit_from_status(status);
}
`;

async function copyTree(source: string, destination: string): Promise<void> {
  const info = await Deno.lstat(source);

  if (info.isDirectory) {
    await Deno.mkdir(destination, { recursive: true });
    for await (const entry of Deno.readDir(source)) {
      await copyTree(
        joinPath(source, entry.name),
        joinPath(destination, entry.name),
      );
    }
    return;
  }

  if (info.isFile) {
    await Deno.copyFile(source, destination);
  }
}

async function digestHex(
  algorithm: "SHA-1" | "SHA-256",
  data: ArrayBuffer,
): Promise<string> {
  const hash = await crypto.subtle.digest(algorithm, data);
  return Array.from(new Uint8Array(hash))
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

async function downloadVerifiedFile(
  url: string,
  destination: string,
  algorithm: "SHA-1" | "SHA-256",
  expectedDigest: string,
): Promise<void> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to download ${url}: HTTP ${response.status}`);
  }

  const data = await response.arrayBuffer();
  const actualDigest = await digestHex(algorithm, data);
  if (actualDigest !== expectedDigest.toLowerCase()) {
    throw new Error(
      `${url} ${algorithm} mismatch: expected ${expectedDigest}, got ${actualDigest}`,
    );
  }

  await Deno.writeFile(destination, new Uint8Array(data));
}

function tauriNsisCacheDirs(): string[] {
  const localAppData = Deno.env.get("LOCALAPPDATA");
  return [
    localAppData ? joinPath(localAppData, "tauri", "NSIS") : "",
    joinPath(repoRoot, "src-tauri", "target", ".tauri", "NSIS"),
  ].filter(Boolean);
}

async function ensureWindowsNsisCache(): Promise<void> {
  if (Deno.build.os !== "windows") {
    return;
  }

  if (!Deno.env.get("SOURCE_DATE_EPOCH")) {
    return;
  }

  const [cacheDir] = tauriNsisCacheDirs();
  if (!cacheDir) {
    return;
  }

  const makensisPath = joinPath(cacheDir, "makensis.exe");
  const utilsPath = joinPath(
    cacheDir,
    "Plugins",
    "x86-unicode",
    "additional",
    "nsis_tauri_utils.dll",
  );

  if ((await exists(makensisPath)) && (await exists(utilsPath))) {
    return;
  }

  const tempDir = await Deno.makeTempDir({ prefix: "sonar-tauri-nsis-" });
  try {
    const nsisZipPath = joinPath(tempDir, `nsis-${tauriNsisVersion}.zip`);
    const extractDir = joinPath(tempDir, "extract");

    await downloadVerifiedFile(
      tauriNsisZipUrl,
      nsisZipPath,
      "SHA-256",
      tauriNsisZipSha256,
    );
    await Deno.mkdir(extractDir, { recursive: true });
    await runCommand("powershell", [
      "-NoProfile",
      "-ExecutionPolicy",
      "Bypass",
      "-Command",
      "$ErrorActionPreference = 'Stop'; Expand-Archive -LiteralPath $env:SONAR_NSIS_ZIP_PATH -DestinationPath $env:SONAR_NSIS_EXTRACT_DIR -Force",
    ], {
      env: {
        SONAR_NSIS_ZIP_PATH: nsisZipPath,
        SONAR_NSIS_EXTRACT_DIR: extractDir,
      },
    });

    await removeIfExists(cacheDir);
    await copyTree(joinPath(extractDir, `nsis-${tauriNsisVersion}`), cacheDir);

    await Deno.mkdir(dirnamePath(utilsPath), { recursive: true });
    await downloadVerifiedFile(
      tauriNsisUtilsUrl,
      utilsPath,
      "SHA-1",
      tauriNsisUtilsSha1,
    );

    console.log(`Primed Tauri NSIS cache at ${cacheDir}.`);
  } finally {
    await removeIfExists(tempDir);
  }
}

async function installWindowsMakensisWrapper(): Promise<void> {
  if (Deno.build.os !== "windows") {
    return;
  }

  if (!Deno.env.get("SOURCE_DATE_EPOCH")) {
    return;
  }

  const candidates = tauriNsisCacheDirs().map((dir) =>
    joinPath(dir, "makensis.exe")
  );

  for (const makensisPath of candidates) {
    if (!(await exists(makensisPath))) {
      continue;
    }

    const wrapperDir = dirnamePath(makensisPath);
    const realWrapperDir = joinPath(dirnamePath(wrapperDir), "NSIS-real");
    const realMakensisPath = joinPath(realWrapperDir, "makensis.exe");

    if (!(await exists(realMakensisPath))) {
      await copyTree(wrapperDir, realWrapperDir);

      const legacyRealMakensisPath = joinPath(wrapperDir, "makensis-real.exe");
      if (await exists(legacyRealMakensisPath)) {
        await Deno.copyFile(legacyRealMakensisPath, realMakensisPath);
      }
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
await ensureWindowsNsisCache();
await installWindowsMakensisWrapper();
