import { useEffect, useState, type DragEvent } from 'react';
import {
  canDropVfsItems,
  dropVfsItems,
  endVfsDrag,
  isVfsDrag,
  type VfsDropTarget,
} from './vfs-drag';
import { useGame } from './game-store';

function blockedSurface(event: DragEvent<HTMLElement>, target: VfsDropTarget | null) {
  const origin = event.target instanceof Element ? event.target : null;
  if (origin?.closest('[aria-modal="true"], .kali-menu, .desktop-context-menu')) {
    return true;
  }
  const control = origin?.closest('button, input, textarea, select, .top-panel, .desktop-note');
  return target?.kind === 'folder' && !!control && control !== event.currentTarget;
}

export function useVfsDrop(
  onError: (message: string) => void = (error) => useGame.setState({ error }),
) {
  const [over, setOver] = useState('');
  useEffect(() => {
    const clear = () => setOver('');
    document.addEventListener('dragend', clear);
    document.addEventListener('drop', clear, true);
    return () => {
      document.removeEventListener('dragend', clear);
      document.removeEventListener('drop', clear, true);
    };
  }, []);
  return (target: VfsDropTarget | null) => {
    const key = !target
      ? 'blocked'
      : target.kind === 'folder'
        ? target.path
        : target.kind === 'app'
          ? (target.windowId ?? target.app)
          : 'trash';
    return {
      'data-drop-active': over === key || undefined,
      onDragOver: (event: DragEvent<HTMLElement>) => {
        if (!isVfsDrag(event.dataTransfer)) {
          return;
        }
        event.preventDefault();
        event.stopPropagation();
        const accepted =
          !blockedSurface(event, target) && canDropVfsItems(event.dataTransfer, target);
        event.dataTransfer.dropEffect = accepted
          ? target?.kind === 'app'
            ? 'copy'
            : 'move'
          : 'none';
        setOver(accepted ? key : '');
      },
      onDragLeave: (event: DragEvent<HTMLElement>) => {
        if (
          !(event.relatedTarget instanceof Node) ||
          !event.currentTarget.contains(event.relatedTarget)
        ) {
          setOver('');
        }
      },
      onDrop: (event: DragEvent<HTMLElement>) => {
        if (!isVfsDrag(event.dataTransfer)) {
          return;
        }
        event.preventDefault();
        event.stopPropagation();
        setOver('');
        if (!target || blockedSurface(event, target)) {
          event.dataTransfer.dropEffect = 'none';
          endVfsDrag();
          return;
        }
        const result = dropVfsItems(event.dataTransfer, target);
        endVfsDrag();
        void result.catch((error: unknown) => onError(String(error).replace(/^Error:\s*/, '')));
      },
    };
  };
}
