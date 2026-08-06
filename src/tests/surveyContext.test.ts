import { deepStrictEqual as assertEquals } from 'node:assert/strict';

import {
  isSurveyContextEmpty,
  needsQualification,
} from '../utils/surveyContextCore.ts';

Deno.test('needsQualification - un relevé sans contexte est proposé à la qualification', () => {
  assertEquals(needsQualification({}), true);
  assertEquals(needsQualification(undefined), true);
  assertEquals(needsQualification(null), true);
});

Deno.test('needsQualification - un seul champ renseigné suffit à considérer le relevé qualifié', () => {
  assertEquals(needsQualification({ site: 'Usine Nord' }), false);
  assertEquals(needsQualification({ sensor: 'sonde-A' }), false);
  assertEquals(needsQualification({ interface: 'eth0' }), false);
});

Deno.test('needsQualification - ne repose pas la question une fois le contexte saisi', () => {
  const qualified = { site: 'Poste 63 kV', sensor: 'sonde-B', interface: 'eth1' };
  assertEquals(
    needsQualification(qualified),
    false,
    "arrêter puis enregistrer ne doit pas demander deux fois la même chose",
  );
});

Deno.test('isSurveyContextEmpty - une saisie blanche vaut « non renseigné »', () => {
  // Alignement avec `SurveyContext::normalized_field` côté Rust : sans lui,
  // trois espaces « qualifieraient » définitivement un relevé qui ne l'est pas.
  assertEquals(isSurveyContextEmpty({ site: '   ' }), true);
  assertEquals(isSurveyContextEmpty({ site: '', sensor: '\t', interface: '\n' }), true);
  assertEquals(isSurveyContextEmpty({ site: ' Usine ' }), false);
});

Deno.test('isSurveyContextEmpty - null et undefined sont traités comme absents', () => {
  assertEquals(isSurveyContextEmpty({ site: null, sensor: null, interface: null }), true);
  assertEquals(
    isSurveyContextEmpty({ site: undefined, sensor: undefined, interface: undefined }),
    true,
  );
});
