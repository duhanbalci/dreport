let tooltipEl: HTMLDivElement | null = null
let currentTarget: HTMLElement | null = null

function getTooltip(): HTMLDivElement {
  if (!tooltipEl) {
    tooltipEl = document.createElement('div')
    tooltipEl.className = 'prop-tooltip'
    document.body.appendChild(tooltipEl)
  }
  return tooltipEl
}

function show(el: HTMLElement) {
  const text = el.getAttribute('data-tip')
  if (!text) return

  currentTarget = el
  const tip = getTooltip()
  tip.textContent = text

  // Position before showing so we can measure
  tip.style.top = '0px'
  tip.style.left = '0px'
  tip.classList.add('prop-tooltip--visible')

  const rect = el.getBoundingClientRect()
  const tipRect = tip.getBoundingClientRect()

  let top = rect.top - tipRect.height - 6
  let left = rect.left + rect.width / 2 - tipRect.width / 2

  // Clamp to viewport
  if (top < 4) top = rect.bottom + 6
  if (left < 4) left = 4
  if (left + tipRect.width > window.innerWidth - 4) {
    left = window.innerWidth - tipRect.width - 4
  }

  tip.style.top = `${top}px`
  tip.style.left = `${left}px`
}

function hide() {
  currentTarget = null
  if (tooltipEl) {
    tooltipEl.classList.remove('prop-tooltip--visible')
  }
}

function closest(el: EventTarget | null): HTMLElement | null {
  if (!(el instanceof HTMLElement)) return null
  return el.closest('[data-tip]')
}

let installed = false

/** Call once to enable data-tip tooltips globally via event delegation. */
export function setupTooltips() {
  if (installed) return
  installed = true
  document.addEventListener(
    'pointerenter',
    (e) => {
      const target = closest(e.target)
      if (target) show(target)
    },
    true,
  )

  document.addEventListener(
    'pointerleave',
    (e) => {
      const target = closest(e.target)
      if (target && target === currentTarget) hide()
    },
    true,
  )

  document.addEventListener(
    'focusin',
    (e) => {
      const target = closest(e.target)
      if (target) show(target)
    },
    true,
  )

  document.addEventListener(
    'focusout',
    (e) => {
      const target = closest(e.target)
      if (target && target === currentTarget) hide()
    },
    true,
  )
}
