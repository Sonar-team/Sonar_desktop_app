import {
  equal as assertEquals,
  match as assertMatch,
  ok as assert,
} from "node:assert/strict";
import buildVersions from "../../config/build-versions.env" with {
  type: "text",
};
import dockerfile from "../../Dockerfile" with { type: "text" };
import reproWorkflow from "../../.github/workflows/repro-container.yml" with {
  type: "text",
};
import reproScript from "../../script/release/repro-build-container.sh" with {
  type: "text",
};
import windowsHashComparator from "../../script/ci/compare-repro-windows-hashes.sh" with {
  type: "text",
};
import denyConfig from "../../src-tauri/deny.toml" with { type: "text" };

function parseBuildVersions(source: string): Map<string, string> {
  const versions = new Map<string, string>();
  for (const rawLine of source.split("\n")) {
    const line = rawLine.trim();
    if (line === "" || line.startsWith("#")) continue;

    const separator = line.indexOf("=");
    assert(separator > 0, `Ligne build-versions invalide : ${line}`);
    const name = line.slice(0, separator);
    const value = line.slice(separator + 1).replace(/^"|"$/g, "");
    versions.set(name, value);
  }
  return versions;
}

Deno.test("cargo-deny refuse les versions wildcard", () => {
  assertMatch(denyConfig, /^wildcards = "deny"$/m);
  assertEquals(/^wildcards = "allow"$/m.test(denyConfig), false);
});

Deno.test("les pins xwin du Dockerfile suivent la source de vérité", () => {
  const versions = parseBuildVersions(buildVersions);
  assert(
    dockerfile.startsWith(
      `# syntax=docker/dockerfile:${versions.get("DOCKERFILE_FRONTEND_VERSION")}@${versions.get("DOCKERFILE_FRONTEND_DIGEST")}`,
    ),
    "le frontend Dockerfile doit être verrouillé par version et digest",
  );
  assertEquals(/^# syntax=docker\/dockerfile:1$/m.test(dockerfile), false);
  const dockerArgs = [
    "WINDOWS_CROSS_APT_PACKAGES",
    "CARGO_XWIN_VERSION",
    "XWIN_VERSION",
    "XWIN_SDK_VERSION",
    "XWIN_CRT_VERSION",
    "XWIN_ARCH",
    "XWIN_HTTP_RETRIES",
  ];

  for (const name of dockerArgs) {
    const value = versions.get(name);
    assert(value, `${name} absent de config/build-versions.env`);
    const rendered = name.endsWith("_APT_PACKAGES")
      ? `ARG ${name}="${value}"`
      : `ARG ${name}=${value}`;
    assert(
      dockerfile.includes(rendered),
      `${name} dérive entre Dockerfile et config/build-versions.env`,
    );
  }

  assertMatch(versions.get("XWIN_SDK_VERSION") ?? "", /^\d+\.\d+\.\d+$/);
  assertMatch(versions.get("XWIN_CRT_VERSION") ?? "", /^\d+\.\d+\.\d+\.\d+$/);
  const toolchainStage = dockerfile.indexOf(
    "FROM build-base AS windows-toolchain",
  );
  const rustupPin = dockerfile.indexOf(
    `ENV RUSTUP_TOOLCHAIN="${versions.get("RUST_VERSION")}"`,
  );
  const targetInstall = dockerfile.indexOf(
    "RUN rustup target add x86_64-pc-windows-msvc",
  );
  assert(toolchainStage >= 0);
  assert(
    toolchainStage < rustupPin,
    "le pin RUSTUP_TOOLCHAIN doit apparaître après le stage windows-toolchain",
  );
  assert(
    rustupPin < targetInstall,
    "l'installation de la cible x86_64-pc-windows-msvc doit suivre le pin RUSTUP_TOOLCHAIN",
  );
  assert(
    dockerfile.includes("RUN bash /usr/local/bin/cache-xwin-toolchain.sh"),
  );
  assert(dockerfile.includes("RUN --network=none deno run"));
});

Deno.test("la preuve Windows cross compare deux runners et accepte un hash local", () => {
  assert(reproWorkflow.includes("machine: [machine-a, machine-b]"));
  assert(reproWorkflow.includes("windows_reference_sha256:"));
  assert(
    reproWorkflow.includes(
      "./script/ci/compare-repro-windows-hashes.sh repro-hashes-windows-cross",
    ),
  );
  assert(
    reproScript.includes("args+=(--allow network.host)"),
    "le contournement réseau host doit autoriser l'entitlement buildx",
  );
  assert(
    reproScript.includes('args=(--platform "$docker_platform"'),
    "la preuve doit fixer la plateforme hôte Docker",
  );
  for (const machine of ["machine-a", "machine-b"]) {
    assert(
      windowsHashComparator.includes(machine),
      `le comparateur doit exiger l'artifact ${machine}`,
    );
  }
  assert(
    windowsHashComparator.includes("${#manifests[@]} != 4"),
    "le comparateur doit exiger les quatre builds inter-machines",
  );
});
