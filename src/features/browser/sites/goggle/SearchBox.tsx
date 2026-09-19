import { useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import { z } from 'zod';
import { request } from '../../../../lib/api';
import { useGame } from '../../../../lib/game-store';
import { GoggleIcon } from './GoggleIcon';
import { SearchSuggestions } from './SearchSuggestions';
import { VirtualKeyboard } from './VirtualKeyboard';
import { ImageSearch } from './ImageSearch';
import { VoiceSearch } from './VoiceSearch';

export function SearchBox({
  initialQuery = '',
  compact = false,
  search,
  imageSearch,
}: {
  initialQuery?: string;
  compact?: boolean;
  search: (query: string, lucky?: boolean) => void;
  imageSearch: (source: string) => void;
}) {
  const [query, setQuery] = useState(initialQuery);
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [focused, setFocused] = useState(false);
  const [selected, setSelected] = useState(-1);
  const [panel, setPanel] = useState('');
  const [suggestionError, setSuggestionError] = useState('');
  const input = useRef<HTMLInputElement>(null);
  const caret = useRef<[number, number]>([initialQuery.length, initialQuery.length]);
  const pendingCaret = useRef<number | null>(null);
  const id = useId();
  const revision = useGame((state) => state.revision);
  useEffect(() => {
    setQuery(initialQuery);
  }, [initialQuery]);
  useLayoutEffect(() => {
    if (pendingCaret.current !== null) {
      input.current?.focus();
      input.current?.setSelectionRange(pendingCaret.current, pendingCaret.current);
      pendingCaret.current = null;
    }
  }, [query]);
  useEffect(() => {
    let active = true;
    setSuggestions([]);
    setSelected(-1);
    setSuggestionError('');
    if (!query.trim() || !focused) {
      return;
    }
    const timer = window.setTimeout(() => {
      void request('search_suggestions', { query }, z.array(z.string()))
        .then((values) => {
          if (active) {
            setSuggestions(values);
          }
        })
        .catch((e: unknown) => {
          if (active) {
            setSuggestionError(String(e));
          }
        });
    }, 120);
    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [query, focused, revision]);
  const submit = (value = query, lucky = false) => {
    setFocused(false);
    setPanel('');
    setQuery(value);
    search(value.trim(), lucky);
  };
  const edit = (text: string, remove = false) => {
    const [start, end] = caret.current;
    const from = remove && start === end ? Math.max(0, start - 1) : start;
    const next = (query.slice(0, from) + text + query.slice(end)).slice(0, 200);
    setQuery(next);
    const position = Math.min(from + text.length, next.length);
    caret.current = [position, position];
    pendingCaret.current = position;
  };
  const showSuggestions = focused && suggestions.length > 0 && panel !== 'keyboard';
  return (
    <div className={`goggle-search-box ${compact ? 'goggle-search-compact' : ''}`}>
      <form
        role="search"
        onSubmit={(e) => {
          e.preventDefault();
          submit(selected >= 0 && showSuggestions ? suggestions[selected] : query);
        }}
      >
        <div className={`goggle-search-field ${showSuggestions ? 'goggle-with-suggestions' : ''}`}>
          <GoggleIcon name="search" />
          <input
            ref={input}
            aria-label="Pesquisar no Goggle"
            role="combobox"
            aria-autocomplete="list"
            aria-expanded={showSuggestions}
            aria-controls={showSuggestions ? id : undefined}
            aria-activedescendant={
              showSuggestions && selected >= 0 ? `${id}-${selected}` : undefined
            }
            autoComplete="off"
            spellCheck={false}
            maxLength={200}
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              caret.current = [e.target.selectionStart ?? 0, e.target.selectionEnd ?? 0];
            }}
            onSelect={(e) => {
              const element = e.currentTarget;
              caret.current = [element.selectionStart ?? 0, element.selectionEnd ?? 0];
            }}
            onFocus={() => setFocused(true)}
            onBlur={() => setFocused(false)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.nativeEvent.isComposing) {
                e.preventDefault();
                submit(selected >= 0 && showSuggestions ? suggestions[selected] : query);
              }
              if (e.key === 'Escape') {
                e.stopPropagation();
                setFocused(false);
                setPanel('');
              }
              if (showSuggestions && (e.key === 'ArrowDown' || e.key === 'ArrowUp')) {
                e.preventDefault();
                setSelected((old) =>
                  old < 0
                    ? e.key === 'ArrowDown'
                      ? 0
                      : suggestions.length - 1
                    : (old + (e.key === 'ArrowDown' ? 1 : -1) + suggestions.length) %
                      suggestions.length,
                );
              }
            }}
          />
          {query && (
            <button
              type="button"
              className="goggle-icon-button goggle-clear"
              aria-label="Limpar pesquisa"
              onClick={() => {
                setQuery('');
                caret.current = [0, 0];
                input.current?.focus();
              }}
            >
              <GoggleIcon name="close" />
            </button>
          )}
          <button
            type="button"
            className="goggle-icon-button"
            aria-label="Abrir teclado virtual"
            aria-expanded={panel === 'keyboard'}
            onMouseDown={(e) => e.preventDefault()}
            onClick={() => setPanel(panel === 'keyboard' ? '' : 'keyboard')}
          >
            <GoggleIcon name="keyboard" />
          </button>
          <button
            type="button"
            className="goggle-icon-button goggle-color-icon"
            aria-label="Pesquisar por voz"
            onClick={() => setPanel('voice')}
          >
            <GoggleIcon name="mic" />
          </button>
          <button
            type="button"
            className="goggle-icon-button goggle-color-icon"
            aria-label="Pesquisar por imagem"
            onClick={() => setPanel('image')}
          >
            <GoggleIcon name="image" />
          </button>
          {compact && (
            <button
              type="submit"
              className="goggle-icon-button goggle-color-icon"
              aria-label="Pesquisar Goggle"
            >
              <GoggleIcon name="search" />
            </button>
          )}
        </div>
        {showSuggestions && (
          <SearchSuggestions
            id={id}
            values={suggestions}
            selected={selected}
            choose={(value) => submit(value)}
          />
        )}
        {!compact && (
          <div className="goggle-search-actions">
            <button type="submit">Pesquisar Goggle</button>
            <button type="button" onClick={() => submit(query, true)}>
              Estou com sorte
            </button>
          </div>
        )}
      </form>
      {suggestionError && focused && (
        <p role="status" className="goggle-note">
          Sugestões indisponíveis. {suggestionError}
        </p>
      )}
      {panel === 'keyboard' && (
        <VirtualKeyboard
          insert={(text) => edit(text)}
          backspace={() => edit('', true)}
          submit={() => submit()}
          close={() => setPanel('')}
        />
      )}
      {panel === 'voice' && (
        <VoiceSearch close={() => setPanel('')} search={(value) => submit(value)} />
      )}
      {panel === 'image' && (
        <ImageSearch
          close={() => setPanel('')}
          search={(source) => {
            setPanel('');
            imageSearch(source);
          }}
        />
      )}
    </div>
  );
}
