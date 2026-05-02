import { Bench } from 'tinybench'
import { TreRegex } from '../index.js'

// -----------------------------------------------------------------------------
// Benchmark Setup
// -----------------------------------------------------------------------------

const bench = new Bench({ time: 1000 })

// The dataset we will search against
const tinyText = 'I ate an apple today.'
const mediumText = 'The quick brown fox jumps over the lazy dog. '.repeat(10) // ~450 chars
const massiveText = 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. '.repeat(100) // ~5500 chars

// Native JS RegExps
const nativeExact = new RegExp('apple')
const nativeComplex = new RegExp('(\\w+)\\s+(\\w+)')

// TRE RegExps
const treExact = new TreRegex('apple')
const treComplex = new TreRegex('(\\w+)\\s+(\\w+)')

// -----------------------------------------------------------------------------
// 1. Exact Matching (JS RegExp vs TreRegex)
// -----------------------------------------------------------------------------

bench
  .add('Native JS RegExp - Exact Match (Short Text)', () => {
    nativeExact.exec(tinyText)
  })
  .add('TreRegex       - Exact Match (Short Text)', () => {
    treExact.exec(tinyText)
  })
  .add('Native JS RegExp - Exact Match (Massive Text)', () => {
    nativeExact.exec(massiveText)
  })
  .add('TreRegex       - Exact Match (Massive Text)', () => {
    treExact.exec(massiveText)
  })

// -----------------------------------------------------------------------------
// 2. Fuzzy Matching (TreRegex Only)
// -----------------------------------------------------------------------------
// We benchmark how the engine scales as we introduce fuzzy limits and costs.

bench
  .add('TreRegex       - Fuzzy Match (1 error limit)', () => {
    treExact.exec(tinyText, { maxErrors: 1 })
  })
  .add('TreRegex       - Fuzzy Match (3 error limit)', () => {
    treExact.exec(tinyText, { maxErrors: 3 })
  })
  .add('TreRegex       - Fuzzy Match (Granular weights/costs)', () => {
    treExact.exec(tinyText, {
      maxCost: 2,
      weightInsertion: 1,
      weightDeletion: 1,
      weightSubstitution: 2,
    })
  })

// -----------------------------------------------------------------------------
// 3. Capture Groups
// -----------------------------------------------------------------------------

bench
  .add('Native JS RegExp - Capture Groups', () => {
    nativeComplex.exec(mediumText)
  })
  .add('TreRegex       - Capture Groups', () => {
    treComplex.exec(mediumText)
  })

// -----------------------------------------------------------------------------
// 4. matchAll Iteration
// -----------------------------------------------------------------------------

bench
  .add('Native JS RegExp - matchAll', () => {
    const globalRegexp = new RegExp('fox', 'g')
    Array.from(mediumText.matchAll(globalRegexp))
  })
  .add('TreRegex       - matchAll', () => {
    const treFox = new TreRegex('fox')
    treFox.matchAll(mediumText)
  })
  .add('TreRegex       - matchAll (Fuzzy)', () => {
    const treFox = new TreRegex('fox')
    treFox.matchAll(mediumText, { maxErrors: 1 })
  })

// -----------------------------------------------------------------------------
// Runner
// -----------------------------------------------------------------------------

async function run() {
  console.log('Running benchmarks...\n')
  await bench.run()

  // 1. Sort the tasks from fastest to slowest (Descending order)
  const sortedTasks = [...bench.tasks].sort((a, b) => {
    const hzA = (a.result as any)?.throughput?.mean ?? 0
    const hzB = (b.result as any)?.throughput?.mean ?? 0
    return hzB - hzA
  })

  console.table(
    sortedTasks.map((task, index) => {
      const result = task.result as any
      const cleanName = task.name.trim().replace(/\s+/g, ' ')

      // Handle failed tasks
      if (result?.state === 'errored' || !result?.throughput) {
        return {
          '#': '❌',
          'Task Name': cleanName,
          'Ops/sec': 'FAILED',
          Margin: 'N/A',
          'Drop from Previous': 'N/A',
        }
      }

      const hz = result.throughput.mean
      const rme = result.throughput.rme

      let relativeSpeed = '🏆 Absolute Fastest'

      // 2. Compare against the task immediately preceding it
      if (index > 0) {
        const prevTask = sortedTasks[index - 1]
        const prevHz = (prevTask.result as any).throughput.mean

        // Calculate percentage drop
        const percentSlower = ((prevHz - hz) / prevHz) * 100

        relativeSpeed = `⬇ ${percentSlower.toFixed(2)}%`
      }

      return {
        '#': `${index + 1}`,
        'Task Name': cleanName,
        'Ops/sec': Math.round(hz).toLocaleString(),
        Margin: `±${rme.toFixed(2)}%`,
        'Drop from Previous': relativeSpeed,
      }
    }),
  )
}

run().catch(console.error)
