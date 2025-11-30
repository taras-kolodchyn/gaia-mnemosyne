let listener: ((msg: string) => void) | null = null

export function registerToastListener(fn: (msg: string) => void) {
  listener = fn
}

export function emitToast(message: string) {
  if (listener) listener(message)
}
