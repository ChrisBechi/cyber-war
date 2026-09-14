export async function saveInstallerCapture(screen: HTMLElement) {
  const clone = screen.cloneNode(true) as HTMLElement;
  const originals = screen.querySelectorAll('input');
  clone.querySelectorAll('input').forEach((input, index) => {
    input.setAttribute(
      'value',
      originals[index].type === 'password' ? '••••••' : originals[index].value,
    );
    if (originals[index].checked) {
      input.setAttribute('checked', '');
    }
  });
  const blob = await fetch('/assets/kali/installer-reference.png').then((r) => r.blob());
  const background = await new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(typeof reader.result === 'string' ? reader.result : '');
    reader.onerror = reject;
    reader.readAsDataURL(blob);
  });
  const styles = Array.from(document.styleSheets)
    .flatMap((sheet) => Array.from(sheet.cssRules).map((rule) => rule.cssText))
    .join('\n');
  const { width, height } = screen.getBoundingClientRect();
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}"><foreignObject width="100%" height="100%"><div xmlns="http://www.w3.org/1999/xhtml"><style>${styles.replaceAll('/assets/kali/installer-reference.png', background)}</style>${new XMLSerializer().serializeToString(clone)}</div></foreignObject></svg>`;
  const url = URL.createObjectURL(new Blob([svg], { type: 'image/svg+xml' }));
  const link = document.createElement('a');
  link.href = url;
  link.download = `kali-instalacao-${screen.dataset.stage ?? 'tela'}.svg`;
  link.click();
  window.setTimeout(() => URL.revokeObjectURL(url), 1000);
}
