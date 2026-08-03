import { deepStrictEqual as assertEquals } from 'node:assert/strict';

import {
  dirnameOf,
  MAX_RECENT_PROJECTS,
  pushRecent,
} from '../utils/recentProjectsCore.ts';

Deno.test('pushRecent - ajoute en tête d\'une liste vide', () => {
  assertEquals(pushRecent([], '/a/p1.sonar'), ['/a/p1.sonar']);
});

Deno.test('pushRecent - le plus récent passe devant', () => {
  assertEquals(pushRecent(['/a/p1.sonar'], '/a/p2.sonar'), [
    '/a/p2.sonar',
    '/a/p1.sonar',
  ]);
});

Deno.test('pushRecent - un chemin déjà présent remonte sans doublon', () => {
  assertEquals(
    pushRecent(['/a/p2.sonar', '/a/p1.sonar'], '/a/p1.sonar'),
    ['/a/p1.sonar', '/a/p2.sonar'],
  );
});

Deno.test('pushRecent - la liste est plafonnée', () => {
  const full = Array.from(
    { length: MAX_RECENT_PROJECTS },
    (_, i) => `/a/p${i}.sonar`,
  );
  const result = pushRecent(full, '/a/nouveau.sonar');
  assertEquals(result.length, MAX_RECENT_PROJECTS);
  assertEquals(result[0], '/a/nouveau.sonar');
  assertEquals(
    result.includes(`/a/p${MAX_RECENT_PROJECTS - 1}.sonar`),
    false,
    'le plus ancien est évincé',
  );
});

Deno.test('dirnameOf - chemins POSIX et Windows', () => {
  assertEquals(dirnameOf('/home/user/audit.sonar'), '/home/user');
  assertEquals(dirnameOf('C:\\relevés\\audit.sonar'), 'C:\\relevés');
  assertEquals(dirnameOf('audit.sonar'), undefined);
  assertEquals(dirnameOf('/audit.sonar'), undefined);
});
