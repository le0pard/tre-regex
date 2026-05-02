# @tre-regex/regex

`@tre-regex/regex` provides a high-performance Node.js interface to the [TRE](https://github.com/laurikari/tre) C library. It brings robust approximate (fuzzy) regular expression matching to JavaScript and TypeScript, featuring multi-byte Unicode string safety, and granular error limits.

## Why?

Standard JavaScript `RegExp` expressions are strictly exact. If you are searching text containing typos, OCR errors, or variations in spelling, standard regex will fail.

While string distance metrics (like Levenshtein distance) exist in the JS ecosystem, they usually require comparing whole strings against other whole strings. `@tre-regex/regex` solves this by allowing you to search for a pattern *within* a larger body of text while permitting a configurable number of errors (insertions, deletions, and substitutions).

## Features

* **Approximate Matching**: Find matches even if the target string has missing, extra, or substituted characters.
* **Granular Control**: Set strict limits on `maxErrors`, or fine-tune by specific error types (`maxInsertions`, `maxDeletions`, `maxSubstitutions`).
* **Multi-byte Unicode Safety**: Transparently maps underlying C byte-offsets back to native JavaScript UTF-16 character indices (e.g., emojis won't break your offsets or `String.prototype.slice`).

## Installation

You can install the package using your preferred package manager. Pre-built binaries are provided for most major operating systems and architectures.

```bash
yarn add @tre-regex/regex
# or
npm install @tre-regex/regex
```

## Usage

### Basic Matching

Create a new `TreRegex` object and use `exec` or `test` to search text.

```javascript
import { TreRegex } from '@tre-regex/regex';

// The second parameter is an optional boolean for ignoreCase
const regex = new TreRegex('apple', true);

// Simple boolean check
regex.test('I ate an APPLE today');
// => true

// Get detailed match data
const result = regex.exec('I ate an apple today');
/* => {
      matchText: "apple",
      submatches: [],
      index: 9,
      endIndex: 14,
      cost: 0,
      errors: { insertions: 0, deletions: 0, substitutions: 0 }
    }
*/
```

### Fuzzy Matching

You can configure fuzziness by passing an options object directly to the `exec` method.

```javascript
const regex = new TreRegex('apple');

// Allow up to 1 error of any kind
regex.exec('I ate an aple', { maxErrors: 1 });
// => { matchText: "aple", submatches: [], index: 9, endIndex: 13, cost: 1, errors: { insertions: 0, deletions: 1, substitutions: 0 } }

// Allow substitutions, but explicitly forbid deletions
regex.exec('I ate an aple', { maxSubstitutions: 1, maxDeletions: 0 });
// => undefined
```

### Finding All Matches

Use `matchAll` to find every occurrence of a pattern in a string. It returns an array of match objects.

```javascript
const regex = new TreRegex('cat');

regex.matchAll('cat, cot, cut', { maxErrors: 1 });
/* => [
  { matchText: "cat", submatches: [], index: 0, endIndex: 3, cost: 0, errors: { insertions: 0, deletions: 0, substitutions: 0 } },
  { matchText: "cot", submatches: [], index: 5, endIndex: 8, cost: 1, errors: { insertions: 0, deletions: 0, substitutions: 1 } },
  { matchText: "cut", submatches: [], index: 10, endIndex: 13, cost: 1, errors: { insertions: 0, deletions: 0, substitutions: 1 } }
] */
```

### Capture Groups (Submatches)

`TreRegex` fully supports standard POSIX capture groups using parentheses `()`. Whenever a match is found, any captured data is returned as an array of strings under the `submatches` key in the result object.

If your pattern does not contain any capture groups, `submatches` will simply return an empty array `[]`.

```javascript
const regex = new TreRegex('I love (ruby|python|javascript)');
const result = regex.exec('I love javascript a lot');

// The captured group is extracted exactly as it was matched
result.submatches; // => ["javascript"]
```

#### Multiple and Optional Groups

You can define multiple capture groups, and they will be returned in the array in the exact order they appear in the pattern.

If you use an optional capture group `?` that does not end up matching anything in the target text, `TreRegex` will safely insert `undefined` (or `null`) in its place in the array to maintain the correct index order.

```javascript
// The first group (cat) is optional. The second group (dog) is required.
const regex = new TreRegex('(cat)?(dog)');

const result = regex.exec('dog');
// result.submatches => [undefined, "dog"]
```

#### Fuzzy Capture Groups

One of the most powerful features of `TreRegex` is that capture groups respect your fuzzy matching rules! If a typo occurs *inside* a capture group, the `submatches` array will return the actual typed text with the typo included.

```javascript
const regex = new TreRegex('I ate an (apple)');

// We allow 1 error. The user typed 'aple' (1 deletion).
const result = regex.exec('I ate an aple', { maxErrors: 1 });

result.submatches; // => ["aple"]
```

#### The 9-Group Limit

For memory safety and performance during the native C-to-Rust bridge, `TreRegex` allocates a strict maximum of 10 slots per match. Because the first slot is always reserved for the full regex match itself, the engine will only extract a maximum of **9 capture groups** per match.

If your pattern contains 10 or more capture groups `()`, the regex will still compile and match perfectly, but any captured groups beyond the 9th one will be safely ignored and omitted from the `submatches` array.

## Configuration Options

`TreRegex` provides fine-grained control over how patterns are compiled and how fuzzy matching constraints are applied.

### Initialization Options

When creating a new `TreRegex` instance, the constructor takes the pattern as the first argument, and an optional `ignoreCase` boolean as the second:

```javascript
// Fails because case doesn't match
const exactRegex = new TreRegex('javascript');
exactRegex.test('JAVASCRIPT'); // => false

// Succeeds using the ignoreCase flag
const caseRegex = new TreRegex('javascript', true);
caseRegex.test('JAVASCRIPT'); // => true
```

### Fuzzy Matching Options

When calling `exec`, `test`, or `matchAll`, you can pass an options object. If no options are provided, `TreRegex` forces an **exact match** (0 errors allowed).

#### Error Limits

These options strictly limit the number of specific operations required to transform the pattern into the matched string.

* **`maxErrors`** *(number)*: The total maximum number of combined errors (insertions + deletions + substitutions) allowed for a match.
* **`maxInsertions`** *(number)*: The maximum number of extra characters allowed in the searched text. *(e.g., Pattern `cat` matching `cart` is 1 insertion)*.
* **`maxDeletions`** *(number)*: The maximum number of missing characters in the searched text. *(e.g., Pattern `cat` matching `ct` is 1 deletion)*.
* **`maxSubstitutions`** *(number)*: The maximum number of swapped characters. *(e.g., Pattern `cat` matching `cot` is 1 substitution)*.

> **Note:** If you specify granular limits (like `maxDeletions: 1`) but omit `maxErrors`, the engine will automatically calculate the maximum allowed errors so you don't accidentally trigger an unlimited fuzzy search.

```javascript
const regex = new TreRegex('banana');

// Allow up to 2 typos of any kind
regex.exec('bananana', { maxErrors: 2 }); // => matches "bananana" (2 insertions)
regex.exec('bnnna', { maxErrors: 2 });    // => matches "bnnna" (2 deletions)
regex.exec('bonono', { maxErrors: 2 });   // => matches "bonono" (2 substitutions)

// Another example
const strictRegex = new TreRegex('library');

// Allow 1 deletion, but STRICTLY 0 substitutions and 0 insertions
strictRegex.exec('librry', { maxDeletions: 1, maxSubstitutions: 0, maxInsertions: 0 });
// => matches "librry"

// This fails because 'lubrary' requires a substitution, which we set to 0
strictRegex.exec('lubrary', { maxDeletions: 1, maxSubstitutions: 0, maxInsertions: 0 });
// => undefined
```

#### Cost and Weights

Instead of hard limits, you can assign different "costs" to different types of errors. This is useful if you want to penalize certain typos more heavily than others.

* **`maxCost`** *(number)*: The maximum total cost allowed for a match to be considered successful.
* **`weightInsertion`** *(number)*: The cost penalty for each inserted character.
* **`weightDeletion`** *(number)*: The cost penalty for each deleted character.
* **`weightSubstitution`** *(number)*: The cost penalty for each substituted character.

```javascript
const regex = new TreRegex('algorithm');

// We allow a maximum cost of 2.
// Missing/extra characters cost 1 point.
// Wrong characters cost 3 points.
const options = {
  maxCost: 2,
  weightDeletion: 1,
  weightInsertion: 1,
  weightSubstitution: 3
};

// 'algoritm' has 1 deletion. Cost = 1. (Passes, 1 < 2)
regex.test('algoritm', options); // => true

// 'algorethm' has 1 substitution. Cost = 3. (Fails, 3 > 2)
regex.test('algorethm', options); // => false
```

## Gotchas & Best Practices

### The "Empty Match" Phenomenon

Because `TreRegex` relies on strict mathematical edit distances, you must be careful when setting `maxErrors` to a value that is **greater than or equal to the length of your pattern**.

If you allow 3 errors on a 3-letter word, the engine considers *deleting all 3 characters* to be a valid mathematical match (cost = 3). This will result in an unexpected match against an empty string (`""`).

```javascript
const regex = new TreRegex('cat');

// We allow 3 errors on a 3-letter word.
// The engine matches "cow" (2 substitutions)...
// but it also matches "" at the end of the string (3 deletions)!
regex.matchAll('cot, cow', { maxErrors: 3 });
/* => [
  { matchText: "cot", ..., cost: 1, errors: { insertions: 0, deletions: 0, substitutions: 1 } },
  { matchText: "cow", ..., cost: 2, errors: { insertions: 0, deletions: 0, substitutions: 2 } },
  { matchText: "", ..., cost: 3, errors: { insertions: 0, deletions: 3, substitutions: 0 } }
] */
```

**Best Practice**: if you need a high `maxErrors` limit but want to prevent the engine from matching empty strings, explicitly cap the `maxDeletions` option so that at least one character of your pattern must survive:

```javascript
// Allow 3 total errors, but strictly forbid the engine from deleting more than 2 characters
regex.matchAll('cot, cow', { maxErrors: 3, maxDeletions: 2 });
// The empty match is mathematically prevented and omitted
```

### POSIX vs. PCRE Syntax

JavaScript’s built-in `RegExp` engine uses a PCRE-like syntax, which supports advanced features like lookaheads `(?=...)` and lookbehinds.

The underlying TRE C-library uses **POSIX Extended Regular Expressions (ERE)**. While it supports standard regex features (character classes `[a-z]`, quantifiers `*`, `+`, `?`, and grouping), it **does not** support Perl-specific extensions.

```javascript
// Valid TRE syntax
new TreRegex('(cat|dog)s?');

// INVALID: Lookarounds are not supported by POSIX ERE
new TreRegex('cat(?=s)'); // Throws: Failed to compile regex pattern
```

### The Performance Cost of Extreme Fuzziness

Fuzzy matching is inherently more computationally expensive than exact matching. The TRE algorithm scales based on the length of the string and the number of allowed errors.

If you are searching a massive block of text (like a whole book) and set `maxErrors: 10`, the engine has to calculate an enormous number of branching possibilities.

**Best Practice**: Keep your error limits tight and realistic. An error limit of 1 to 3 is usually perfect for catching typos. If you need to allow a massive number of errors, consider breaking the target text into smaller chunks (like sentences or words) before matching.

### Unicode Character Indices vs. Byte Offsets

In C, strings are just arrays of bytes. An emoji like 🍎 takes up 4 bytes, which often breaks indexing when C-libraries pass data back to high-level languages.

`TreRegex` handles this natively in Rust. The `index` and `endIndex` returned in the match object are strictly mapped back to **JavaScript UTF-16 code units**, not raw C byte offsets.

**Best Practice**: You can safely use the returned indices directly with `String.prototype.slice()`, even if the text is filled with emojis or multi-byte characters!

```javascript
const regex = new TreRegex('apple');
const target = 'I ate 🍎 and an aple';

const result = regex.exec(target, { maxErrors: 1 });
// result.index is 15, result.endIndex is 19

// This is 100% safe and will correctly return "aple"
target.slice(result.index, result.endIndex);
```

### Overlapping Matches in `matchAll`

When using `matchAll`, be aware that the engine consumes the string as it matches. By default, standard regex engines (including TRE) do not return overlapping matches.

If you search for `"ana"` in `"banana"`, it will only match the first `"ana"`. Once it consumes those characters, it moves on to the remaining `"na"`.

```javascript
const regex = new TreRegex('ana');

// Returns 1 match, not 2!
regex.matchAll('banana');
// => [{ matchText: "ana", index: 1, endIndex: 4, ... }]
```

If you need to find overlapping fuzzy matches, you will need to manually step through the string by advancing your starting index by 1 character after each search.

## Development

This package uses [napi-rs](https://napi.rs/) to bridge the TRE C library with Node.js.

To build the native addon locally, you need the Rust toolchain installed on your machine.

```bash
# Install dependencies
yarn install

# Build the native add-on (compiles Rust & C)
yarn build

# Run tests
yarn test
```

## License

This package is available as open source under the terms of the MIT License.
