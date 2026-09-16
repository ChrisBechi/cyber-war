import { createContext } from 'react';

// Applications with an integrated toolbar share the native window drag surface.
export const WindowHeaderContext = createContext<HTMLDivElement | null>(null);
