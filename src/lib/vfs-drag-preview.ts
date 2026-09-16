let preview: HTMLElement | null = null;

export function clearVfsDragPreview() {
  preview?.remove();
  preview = null;
}

export function showVfsDragPreview(data: DataTransfer, source: HTMLElement, count: number) {
  clearVfsDragPreview();
  const ghost = document.createElement('div');
  ghost.className = 'vfs-drag-preview';
  ghost.setAttribute('aria-hidden', 'true');
  const icon = source.querySelector('.app-icon, .archive-icon, svg, img');
  if (icon) {
    ghost.append(icon.cloneNode(true));
  }
  const label = document.createElement('span');
  label.className = 'vfs-drag-preview__name';
  label.textContent =
    source.querySelector('.desktop-shortcut-name, :scope > span:last-child')?.textContent ??
    source.getAttribute('title') ??
    source.textContent;
  ghost.append(label);
  if (count > 1) {
    const badge = document.createElement('b');
    badge.className = 'vfs-drag-preview__count';
    badge.textContent = String(count);
    ghost.append(badge);
  }
  document.body.append(ghost);
  preview = ghost;
  data.setDragImage(ghost, 20, 20);
}
