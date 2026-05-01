import test from 'ava'
import { TreRegex } from '../index.js'

// -----------------------------------------------------------------------------
// Initialization
// -----------------------------------------------------------------------------

test('Initialization - compiles a valid regular expression', (t) => {
  t.notThrows(() => new TreRegex('hello'))
})

test('Initialization - throws an error for an invalid regular expression', (t) => {
  t.throws(() => new TreRegex('(invalid'))
})

test('Initialization - respects the ignore_case flag', (t) => {
  const regex = new TreRegex('hello', true)
  t.true(regex.test('HELLO'))
})

// -----------------------------------------------------------------------------
// Method: #exec
// -----------------------------------------------------------------------------

test('#exec - returns undefined when there is no match', (t) => {
  const regex = new TreRegex('apple')
  t.falsy(regex.exec('banana'))
})

test('#exec - finds an exact match', (t) => {
  const regex = new TreRegex('apple')
  const result = regex.exec('I ate an apple today')

  t.truthy(result)
  t.is(result?.matchText, 'apple')
  t.is(result?.index, 9)
  t.is(result?.endIndex, 14)
  t.is(result?.cost, 0)
})

test('#exec - finds a fuzzy match with substitutions', (t) => {
  const regex = new TreRegex('apple')
  // 'aple' has 1 deletion. 'appple' has 1 insertion. 'ipple' has 1 substitution.
  const result = regex.exec('I ate an ipple today', { maxErrors: 1 })

  t.truthy(result)
  t.is(result?.matchText, 'ipple')
  t.is(result?.cost, 1)
  t.is(result?.errors.substitutions, 1)
})

test('#exec - respects granular fuzzy options', (t) => {
  const regex = new TreRegex('apple')

  // We allow 1 error, but 0 substitutions AND 0 deletions. 'ipple' should fail.
  const result1 = regex.exec('I ate an ipple', { maxErrors: 1, maxSubstitutions: 0, maxDeletions: 0 })
  t.falsy(result1)

  // But a deletion should still work if we allow it
  const result2 = regex.exec('I ate an aple', { maxErrors: 1, maxSubstitutions: 0 })
  t.truthy(result2)
  t.is(result2?.matchText, 'aple')
  t.is(result2?.errors.deletions, 1)
})

test('#exec - handles massive Unicode characters correctly (byte-to-char index mapping)', (t) => {
  const regex = new TreRegex('apple')
  // 👨‍👩‍👧‍👦 is 11 bytes in UTF-8, but 1 character in JS. 🚀 is 4 bytes.
  const unicodeText = '👨‍👩‍👧‍👦 loves 🚀, but hates aple!'

  const result = regex.exec(unicodeText, { maxErrors: 1 })

  t.truthy(result)
  t.is(result?.matchText, 'aple')

  // Test if the JS index extraction perfectly matches native String.prototype.slice
  const extracted = unicodeText.slice(result!.index, result!.endIndex)
  t.is(extracted, 'aple')
})

test('#exec - calculates max_err automatically if only granular limits are provided', (t) => {
  const regex = new TreRegex('apple')

  const resultSuccess = regex.exec('ipple', { maxSubstitutions: 1 })
  t.truthy(resultSuccess)
  t.is(resultSuccess?.matchText, 'ipple')

  // 'bople' requires 2 substitutions ('a'->'b' and 'p'->'o'). Should fail.
  const resultFail = regex.exec('bople', { maxSubstitutions: 1 })
  t.falsy(resultFail)
})

test('#exec - handles searching an empty string gracefully', (t) => {
  const regex = new TreRegex('apple')
  t.falsy(regex.exec(''))
})

// -----------------------------------------------------------------------------
// Method: #test
// -----------------------------------------------------------------------------

test('#test - returns true if a match is found', (t) => {
  const regex = new TreRegex('ruby')
  t.true(regex.test('I love ruby'))
})

test('#test - returns false if no match is found', (t) => {
  const regex = new TreRegex('ruby')
  t.false(regex.test('I love python'))
})

// -----------------------------------------------------------------------------
// Method: #matchAll
// -----------------------------------------------------------------------------

test('#matchAll - returns an array of multiple exact matches', (t) => {
  const regex = new TreRegex('cat')
  const results = regex.matchAll('cat, dog, cat')

  t.true(Array.isArray(results))
  t.is(results.length, 2)
  t.is(results[0].index, 0)
  t.is(results[1].index, 10)
})

test('#matchAll - yields multiple fuzzy matches', (t) => {
  const regex = new TreRegex('cat')
  const results = regex.matchAll('cat, cot, cut', { maxErrors: 1 })

  t.is(results.length, 3)
  t.is(results[0].matchText, 'cat')
  t.is(results[1].matchText, 'cot')
  t.is(results[2].matchText, 'cut')
})

// -----------------------------------------------------------------------------
// Memory Management
// -----------------------------------------------------------------------------

test('Memory Management - handles high volume creation without crashing', (t) => {
  t.notThrows(() => {
    for (let i = 0; i < 5000; i++) {
      const regex = new TreRegex('apple|orange|banana')
      regex.exec('I ate an apple')
    }
  })
})

test('Memory Management - handles very long input strings without buffer overflow', (t) => {
  const longString = 'a'.repeat(1_000_000)
  const regex = new TreRegex('a+')

  const result = regex.exec(longString)
  t.truthy(result)
  t.is(result?.matchText.length, 1_000_000)
})

// -----------------------------------------------------------------------------
// Loop Safety
// -----------------------------------------------------------------------------

test('Loop Safety - prevents infinite loops on zero-width matches', (t) => {
  const regex = new TreRegex('a*')

  // AVA throws an error automatically if a test hangs, this proves the loop breaks!
  const results = regex.matchAll('bb')
  t.true(results.length >= 3) // Should find empty matches at index 0, 1, and 2
})

test('Loop Safety - correctly advances when matches are adjacent', (t) => {
  const regex = new TreRegex('aa')
  const results = regex.matchAll('aaaa')

  t.is(results.length, 2)
  t.is(results[0].index, 0)
  t.is(results[1].index, 2)
})

// -----------------------------------------------------------------------------
// Capture Groups (Submatches)
// -----------------------------------------------------------------------------

test('Capture Groups - extracts a single capture group', (t) => {
  const regex = new TreRegex('I love (ruby|python)')
  const result = regex.exec('I love ruby a lot')

  t.truthy(result)
  t.deepEqual(result?.submatches, ['ruby'])
})

test('Capture Groups - extracts multiple capture groups in order', (t) => {
  const regex = new TreRegex('(\\w+)\\s+(\\w+)')
  const result = regex.exec('hello world')

  t.truthy(result)
  t.is(result?.submatches.length, 2)
  t.deepEqual(result?.submatches, ['hello', 'world'])
})

test('Capture Groups - returns an empty array when no capture groups are defined', (t) => {
  const regex = new TreRegex('just a string')
  const result = regex.exec('just a string')

  t.truthy(result)
  t.deepEqual(result?.submatches, [])
})

test('Capture Groups - extracts capture groups accurately during a fuzzy match', (t) => {
  const regex = new TreRegex('I ate an (apple)')
  const result = regex.exec('I ate an aple', { maxErrors: 1 })

  t.truthy(result)
  t.is(result?.matchText, 'I ate an aple')
  t.deepEqual(result?.submatches, ['aple'])
})

test('Capture Groups - inserts undefined/null for optional capture groups that do not match', (t) => {
  const regex = new TreRegex('(cat)?(dog)')
  const result = regex.exec('dog')

  t.truthy(result)
  t.falsy(result?.submatches[0]) // First group didn't match
  t.is(result?.submatches[1], 'dog')
})

test('Capture Groups - safely ignores capture groups beyond the allocated array limit (9)', (t) => {
  const pattern = Array.from({ length: 10 }, (_, i) => `(${i + 1})`).join('-')
  const regex = new TreRegex(pattern)

  const target = Array.from({ length: 10 }, (_, i) => i + 1).join('-')
  const result = regex.exec(target)

  t.truthy(result)
  t.is(result?.submatches.length, 9)
  t.deepEqual(result?.submatches, ['1', '2', '3', '4', '5', '6', '7', '8', '9'])
})

// -----------------------------------------------------------------------------
// Option Sanitization
// -----------------------------------------------------------------------------

test('Option Sanitization - ignores unknown options and safely defaults to exact matching', (t) => {
  const regex = new TreRegex('apple')

  // @ts-ignore - purposefully forcing a bad property into the object
  const result = regex.exec('I ate an aple', { mexErrors: 1 })
  t.falsy(result)
})

// -----------------------------------------------------------------------------
// Binary Data and Null Byte Safety
// -----------------------------------------------------------------------------

test('Binary Data - safely matches text containing null bytes (\\x00) without truncation', (t) => {
  const text = 'hello\x00world'
  const regex = new TreRegex('world')

  const result = regex.exec(text)
  t.truthy(result)
  t.is(result?.matchText, 'world')
  t.is(result?.index, 6)
})

// -----------------------------------------------------------------------------
// Byte-to-Char Cursor Tracking
// -----------------------------------------------------------------------------

test('Cursor Tracking - advances safely through multi-byte characters without losing sync', (t) => {
  const regex = new TreRegex('apple')
  const text = '🍎 apple 🍌 apple 🍇'

  const results = regex.matchAll(text)

  t.is(results.length, 2)

  // In JS (UTF-16), 🍎 is 2 code units, space is 1. 'apple' starts at index 3.
  t.is(results[0].index, 3)
  t.is(results[0].endIndex, 8)
  t.is(text.slice(results[0].index, results[0].endIndex), 'apple')

  // The next apple starts after 'apple ' (6 units) + '🍌 ' (3 units) -> 3 + 6 + 3 = 12
  t.is(results[1].index, 12)
  t.is(results[1].endIndex, 17)
  t.is(text.slice(results[1].index, results[1].endIndex), 'apple')
})

test('Cursor Tracking - strictly adheres to public API and hides internal byte keys', (t) => {
  const regex = new TreRegex('apple')
  const result = regex.exec('apple')

  t.truthy(result)
  // Ensure no internal pointers or C-byte metrics leaked out
  t.false('byte_index' in result!)
  t.false('byte_end_index' in result!)
})

// -----------------------------------------------------------------------------
// Gotchas and Best Practices
// -----------------------------------------------------------------------------

test('Gotchas - matches an empty string when maxErrors >= pattern length', (t) => {
  const regex = new TreRegex('cat')
  const results = regex.matchAll('cot, cow', { maxErrors: 3 })

  const emptyMatch = results.find((r) => r.matchText === '')

  t.truthy(emptyMatch)
  t.is(emptyMatch?.cost, 3)
  t.is(emptyMatch?.errors.deletions, 3)
})

test('Gotchas - prevents empty matches when explicitly capping maxDeletions', (t) => {
  const regex = new TreRegex('cat')
  const results = regex.matchAll('cot, cow', { maxErrors: 3, maxDeletions: 2 })

  const emptyMatch = results.find((r) => r.matchText === '')

  t.falsy(emptyMatch)
  t.is(results.length, 2)
  t.deepEqual(
    results.map((r) => r.matchText),
    ['cot', 'cow'],
  )
})

test('POSIX vs PCRE - compiles standard POSIX ERE syntax without error', (t) => {
  t.notThrows(() => new TreRegex('(cat|dog)s?'))
})

test('POSIX vs PCRE - throws an error for unsupported PCRE syntax like lookaheads', (t) => {
  t.throws(() => new TreRegex('cat(?=s)'))
})

test('Overlapping Matches - consumes the string and does not return overlapping matches by default', (t) => {
  const regex = new TreRegex('ana')
  const results = regex.matchAll('banana')

  t.is(results.length, 1)
  t.is(results[0].matchText, 'ana')
  t.is(results[0].index, 1)
  t.is(results[0].endIndex, 4)
})
