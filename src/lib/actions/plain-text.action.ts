// Turns off the OS autocorrect, auto-capitalisation, spellcheck and autofill. Every input and
// textarea uses it: typed text stays exactly as typed (names, searches, paths, tags, titles).
// The markdown editors (CodeMirror) have these off already.
export function plainText(node: HTMLInputElement | HTMLTextAreaElement) {
  node.setAttribute('autocorrect', 'off');
  node.setAttribute('autocapitalize', 'off');
  node.setAttribute('autocomplete', 'off');
  node.spellcheck = false;
}
