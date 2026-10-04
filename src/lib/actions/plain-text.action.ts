// For fields whose text must stay exactly as typed (adds, searches, names, paths, tags): turns
// off the OS autocorrect, auto-capitalisation, spellcheck and autofill.
export function plainText(node: HTMLInputElement) {
  node.setAttribute('autocorrect', 'off');
  node.setAttribute('autocapitalize', 'off');
  node.setAttribute('autocomplete', 'off');
  node.spellcheck = false;
}
