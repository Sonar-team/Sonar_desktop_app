// Contrat statique du builder Windows officiel. Le build reel et les acces HSM
// restent verifies sur le poste Windows de release.
import {
  equal as assertEquals,
  match as assertMatch,
  ok as assert,
} from "node:assert/strict";
import script from "../../script/release/win_officiel_builder_hsm_prov.ps1" with {
  type: "text",
};
import documentation from "../../script/release/WIN_OFFICIEL_BUILDER_HSM_PROV.md" with {
  type: "text",
};

Deno.test("builder Windows officiel - source et sortie verrouillees", () => {
  assertMatch(script, /git\", \"tag\", \"-v\"|@\("tag", "-v", \$ReleaseTag\)/);
  assert(
    script.includes('"status", "--porcelain=v1", "--untracked-files=all"'),
  );
  assert(
    script.includes("HEAD ($headCommit) ne correspond pas au commit approuve"),
  );
  assert(
    script.includes("OutputDirectory doit etre situe hors du depot source"),
  );
  assert(script.includes("OutputDirectory existe deja et n'est pas vide"));
});

Deno.test("builder Windows officiel - Authenticode reste dans le HSM", () => {
  assert(script.includes("signCommand = [ordered]@{"));
  assert(script.includes('@("sign", "/v", "/sha1"'));
  assert(script.includes('$signArguments += "/sm"'));
  assert(
    script.includes('@("/tr", $TimestampUrl.AbsoluteUri, "/td", "SHA256")'),
  );
  assert(script.includes('$signArguments += "%1"'));
  assert(script.includes("TimeStamperCertificate"));
  assertEquals(script.includes("--no-sign"), false);
  assertEquals(script.includes("repro-env"), false);
});

Deno.test("builder Windows officiel - provenance SLSA signee et reverifiee", () => {
  assert(script.includes('"--type", "slsaprovenance1"'));
  assert(script.includes('"attest-blob"'));
  assert(script.includes('"verify-blob-attestation"'));
  assert(script.includes('"--private-infrastructure"'));
  assert(script.includes('"--use-signing-config=false"'));
  assert(script.includes("CosignKeyReference doit etre une reference HSM/KMS"));
  assert(script.includes("Preflight de la cle Cosign HSM"));
  assert(script.includes('"SHA256SUMS.sigstore.json"'));
  assert(script.includes("publicKeySha256"));
});

Deno.test("builder Windows officiel - commande operateur documentee", () => {
  for (
    const parameter of [
      "-ReleaseTag",
      "-ExpectedCommit",
      "-AuthenticodeThumbprint",
      "-CertificateStore",
      "-TimestampUrl",
      "-CosignKeyReference",
      "-CosignSecurityKey",
      "-CosignPublicKey",
      "-OutputDirectory",
    ]
  ) {
    assert(
      documentation.includes(parameter),
      `${parameter} absent de la documentation`,
    );
  }
  assert(documentation.includes("VERIFY.txt"));
  assert(documentation.includes("SHA256SUMS.sigstore.json"));
  assert(documentation.includes("Ne jamais le passer en paramètre"));
});
