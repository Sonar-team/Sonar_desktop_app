const epochValue = Deno.env.get("SOURCE_DATE_EPOCH");

if (!epochValue) {
  console.log(
    "SOURCE_DATE_EPOCH is not set; skipping bundle input timestamp normalization.",
  );
  Deno.exit(0);
}

const epoch = Number.parseInt(epochValue, 10);

if (!Number.isFinite(epoch) || epoch < 0) {
  throw new Error(`Invalid SOURCE_DATE_EPOCH: ${epochValue}`);
}

const timestamp = new Date(epoch * 1000);
const repoRoot = new URL("../../", import.meta.url);
let normalized = 0;
let normalizedPeFiles = 0;
const stableCodeViewGuid = new TextEncoder().encode("SONAR-REPRODUCE\0");

function childUrl(directory: URL, name: string): URL {
  return new URL(encodeURIComponent(name), `${directory.href}/`);
}

async function exists(path: URL): Promise<boolean> {
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

async function normalizePath(path: URL): Promise<void> {
  await Deno.utime(path, timestamp, timestamp);
  normalized += 1;
}

function getUint16(view: DataView, offset: number): number | undefined {
  if (offset + 2 > view.byteLength) {
    return undefined;
  }
  return view.getUint16(offset, true);
}

function getUint32(view: DataView, offset: number): number | undefined {
  if (offset + 4 > view.byteLength) {
    return undefined;
  }
  return view.getUint32(offset, true);
}

function setUint32(view: DataView, offset: number, value: number): boolean {
  if (offset + 4 > view.byteLength || view.getUint32(offset, true) === value) {
    return false;
  }

  view.setUint32(offset, value, true);
  return true;
}

async function normalizePeMetadata(path: URL): Promise<void> {
  const data = await Deno.readFile(path);
  if (data.byteLength < 0x40 || data[0] !== 0x4d || data[1] !== 0x5a) {
    return;
  }

  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  const peOffset = getUint32(view, 0x3c);
  if (peOffset === undefined || peOffset + 24 > data.byteLength) {
    return;
  }

  if (
    data[peOffset] !== 0x50 || data[peOffset + 1] !== 0x45 ||
    data[peOffset + 2] !== 0 || data[peOffset + 3] !== 0
  ) {
    return;
  }

  let changed = false;
  const coffHeaderOffset = peOffset + 4;
  const numberOfSections = getUint16(view, coffHeaderOffset + 2) ?? 0;
  const optionalHeaderSize = getUint16(view, coffHeaderOffset + 16) ?? 0;
  const optionalHeaderOffset = coffHeaderOffset + 20;

  changed = setUint32(view, coffHeaderOffset + 4, epoch) || changed;
  changed = setUint32(view, optionalHeaderOffset + 64, 0) || changed;

  const magic = getUint16(view, optionalHeaderOffset);
  const dataDirectoryOffset = magic === 0x20b
    ? optionalHeaderOffset + 112
    : magic === 0x10b
    ? optionalHeaderOffset + 96
    : undefined;

  const sectionsOffset = optionalHeaderOffset + optionalHeaderSize;
  const sections = [];
  for (let index = 0; index < numberOfSections; index += 1) {
    const sectionOffset = sectionsOffset + index * 40;
    if (sectionOffset + 40 > data.byteLength) {
      return;
    }

    const virtualSize = getUint32(view, sectionOffset + 8) ?? 0;
    const virtualAddress = getUint32(view, sectionOffset + 12) ?? 0;
    const rawSize = getUint32(view, sectionOffset + 16) ?? 0;
    const rawPointer = getUint32(view, sectionOffset + 20) ?? 0;
    sections.push({ virtualAddress, virtualSize, rawSize, rawPointer });
  }

  const rvaToFileOffset = (rva: number): number | undefined => {
    for (const section of sections) {
      const sectionSize = Math.max(section.virtualSize, section.rawSize);
      if (
        rva >= section.virtualAddress &&
        rva < section.virtualAddress + sectionSize
      ) {
        return section.rawPointer + rva - section.virtualAddress;
      }
    }
    return undefined;
  };

  if (
    dataDirectoryOffset !== undefined &&
    dataDirectoryOffset + 56 <= data.byteLength
  ) {
    const debugDirectoryOffset = dataDirectoryOffset + 6 * 8;
    const debugRva = getUint32(view, debugDirectoryOffset) ?? 0;
    const debugSize = getUint32(view, debugDirectoryOffset + 4) ?? 0;
    const debugFileOffset = rvaToFileOffset(debugRva);

    if (debugRva !== 0 && debugFileOffset !== undefined) {
      const debugEntryCount = Math.floor(debugSize / 28);
      for (let index = 0; index < debugEntryCount; index += 1) {
        const entryOffset = debugFileOffset + index * 28;
        if (entryOffset + 28 > data.byteLength) {
          break;
        }

        changed = setUint32(view, entryOffset + 4, epoch) || changed;

        const debugType = getUint32(view, entryOffset + 12);
        const debugDataSize = getUint32(view, entryOffset + 16) ?? 0;
        const debugDataPointer = getUint32(view, entryOffset + 24) ?? 0;
        const isCodeView = debugType === 2 && debugDataSize >= 24 &&
          debugDataPointer + debugDataSize <= data.byteLength &&
          data[debugDataPointer] === 0x52 &&
          data[debugDataPointer + 1] === 0x53 &&
          data[debugDataPointer + 2] === 0x44 &&
          data[debugDataPointer + 3] === 0x53;

        if (isCodeView) {
          for (let byte = 0; byte < stableCodeViewGuid.byteLength; byte += 1) {
            const offset = debugDataPointer + 4 + byte;
            if (data[offset] !== stableCodeViewGuid[byte]) {
              data[offset] = stableCodeViewGuid[byte];
              changed = true;
            }
          }

          changed = setUint32(view, debugDataPointer + 20, 1) || changed;
        }
      }
    }
  }

  if (changed) {
    await Deno.writeFile(path, data);
    normalizedPeFiles += 1;
  }
}

async function normalizeTree(path: URL): Promise<void> {
  let info;
  try {
    info = await Deno.lstat(path);
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) {
      return;
    }
    throw error;
  }

  if (info.isSymlink) {
    return;
  }

  if (info.isDirectory) {
    const entries = [];
    for await (const entry of Deno.readDir(path)) {
      entries.push(entry.name);
    }

    entries.sort();

    for (const name of entries) {
      await normalizeTree(childUrl(path, name));
    }
  }

  await normalizePath(path);
}

async function normalizeReleaseBundleInputs(releaseRoot: URL): Promise<void> {
  if (!(await exists(releaseRoot))) {
    return;
  }

  const nestedBundleInputs = new Set(["nsis", "resources", "windows"]);
  const entries = [];
  for await (const entry of Deno.readDir(releaseRoot)) {
    entries.push(entry);
  }

  entries.sort((a, b) => a.name.localeCompare(b.name));

  for (const entry of entries) {
    const path = childUrl(releaseRoot, entry.name);
    if (entry.isSymlink) {
      continue;
    }

    if (entry.isFile) {
      if (/\.(dll|exe)$/i.test(entry.name)) {
        await normalizePeMetadata(path);
      }
      await normalizePath(path);
    } else if (entry.isDirectory && nestedBundleInputs.has(entry.name)) {
      await normalizeTree(path);
    }
  }
}

async function normalizeTargetReleaseInputs(targetRoot: URL): Promise<void> {
  if (!(await exists(targetRoot))) {
    return;
  }

  await normalizeReleaseBundleInputs(childUrl(targetRoot, "release"));

  const entries = [];
  for await (const entry of Deno.readDir(targetRoot)) {
    entries.push(entry);
  }

  entries.sort((a, b) => a.name.localeCompare(b.name));

  for (const entry of entries) {
    if (!entry.isDirectory || entry.isSymlink) {
      continue;
    }

    await normalizeReleaseBundleInputs(
      childUrl(childUrl(targetRoot, entry.name), "release"),
    );
  }
}

const packageInputRoots = [
  "dist",
  "LICENSE.md",
  "src-tauri/icons",
  "src-tauri/resources",
  "src-tauri/windows",
];

for (const relativePath of packageInputRoots) {
  await normalizeTree(new URL(relativePath, repoRoot));
}

await normalizeTargetReleaseInputs(new URL("src-tauri/target", repoRoot));

console.log(
  `Normalized ${normalized} bundle input timestamps and ${normalizedPeFiles} PE files to SOURCE_DATE_EPOCH=${epoch}.`,
);
