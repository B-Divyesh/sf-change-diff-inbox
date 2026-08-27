import { compact, csvEscape, diffWords } from './lib';
import { expect, test } from 'vitest';

test('compacts whitespace and long text', () => {
  expect(compact('  alpha\n beta  ')).toBe('alpha beta');
  expect(compact('x'.repeat(200), 20)).toHaveLength(21);
});

test('marks semantic word additions and removals', () => {
  const diff = diffWords('price 10', 'price 12 now');
  expect(diff.old.some((part) => part.type === 'removed' && part.value === '10')).toBe(true);
  expect(diff.next.some((part) => part.type === 'added' && part.value === '12')).toBe(true);
});

test('escapes CSV cells', () => expect(csvEscape('a"b')).toBe('"a""b"'));
