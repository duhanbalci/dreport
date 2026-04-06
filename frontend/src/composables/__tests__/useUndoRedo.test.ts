import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { ref } from 'vue'
import { useUndoRedo } from '../useUndoRedo'

describe('useUndoRedo', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('starts with initial snapshot', () => {
    const source = ref({ value: 1 })
    const { canUndo, canRedo } = useUndoRedo(source)

    expect(canUndo()).toBe(false) // only 1 snapshot (initial)
    expect(canRedo()).toBe(false)
  })

  it('records snapshot after debounce', async () => {
    const source = ref({ value: 1 })
    const { canUndo } = useUndoRedo(source)

    source.value = { value: 2 }
    await vi.advanceTimersByTimeAsync(350) // debounce = 300ms

    expect(canUndo()).toBe(true)
  })

  it('undo restores previous state', async () => {
    const source = ref({ count: 0 })
    const { undo, canUndo } = useUndoRedo(source)

    source.value = { count: 1 }
    await vi.advanceTimersByTimeAsync(350)

    source.value = { count: 2 }
    await vi.advanceTimersByTimeAsync(350)

    expect(source.value.count).toBe(2)

    undo()
    expect(source.value.count).toBe(1)

    undo()
    expect(source.value.count).toBe(0)
  })

  it('redo restores undone state', async () => {
    const source = ref({ count: 0 })
    const { undo, redo, canRedo } = useUndoRedo(source)

    source.value = { count: 1 }
    await vi.advanceTimersByTimeAsync(350)

    undo()
    expect(source.value.count).toBe(0)
    expect(canRedo()).toBe(true)

    redo()
    expect(source.value.count).toBe(1)
    expect(canRedo()).toBe(false)
  })

  it('new mutation clears redo stack', async () => {
    const source = ref({ v: 'a' })
    const { undo, redo, canRedo } = useUndoRedo(source)

    source.value = { v: 'b' }
    await vi.advanceTimersByTimeAsync(350)

    undo()
    expect(canRedo()).toBe(true)

    // New mutation after undo → clears redo
    source.value = { v: 'c' }
    await vi.advanceTimersByTimeAsync(350)

    expect(canRedo()).toBe(false)
  })

  it('respects maxHistory limit', async () => {
    const source = ref({ n: 0 })
    const { canUndo, undo } = useUndoRedo(source, 3) // max 3 snapshots

    source.value = { n: 1 }
    await vi.advanceTimersByTimeAsync(350)

    source.value = { n: 2 }
    await vi.advanceTimersByTimeAsync(350)

    source.value = { n: 3 }
    await vi.advanceTimersByTimeAsync(350)

    // Stack: [1, 2, 3] (initial 0 was shifted out)
    // 3 snapshots, can undo twice (back to 1)
    undo()
    expect(source.value.n).toBe(2)

    undo()
    expect(source.value.n).toBe(1)

    // Can't undo further (stack has only 1 left)
    expect(canUndo()).toBe(false)
  })

  it('skips duplicate snapshots', async () => {
    const source = ref({ x: 1 })
    const { canUndo } = useUndoRedo(source)

    // Set same value
    source.value = { x: 1 }
    await vi.advanceTimersByTimeAsync(350)

    expect(canUndo()).toBe(false) // no new snapshot since value same
  })

  it('debounces rapid changes into one snapshot', async () => {
    const source = ref({ n: 0 })
    const { undo } = useUndoRedo(source)

    // Rapid changes within debounce window
    source.value = { n: 1 }
    await vi.advanceTimersByTimeAsync(100)
    source.value = { n: 2 }
    await vi.advanceTimersByTimeAsync(100)
    source.value = { n: 3 }
    await vi.advanceTimersByTimeAsync(350) // trigger debounce

    // Only one snapshot recorded (n=3), so one undo goes to initial
    undo()
    expect(source.value.n).toBe(0)
  })

  it('undo with only initial snapshot does nothing', () => {
    const source = ref({ v: 'init' })
    const { undo } = useUndoRedo(source)

    undo() // should not crash
    expect(source.value.v).toBe('init')
  })

  it('redo with empty redo stack does nothing', () => {
    const source = ref({ v: 'init' })
    const { redo } = useUndoRedo(source)

    redo() // should not crash
    expect(source.value.v).toBe('init')
  })
})
