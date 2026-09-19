import { GoggleIcon } from './GoggleIcon';

export function SearchSuggestions({
  id,
  values,
  selected,
  choose,
}: {
  id: string;
  values: string[];
  selected: number;
  choose: (query: string) => void;
}) {
  return (
    <ul role="listbox" id={id} aria-label="Sugestões de pesquisa" className="goggle-suggestions">
      {values.map((value, i) => (
        <li
          role="option"
          id={`${id}-${i}`}
          aria-selected={i === selected}
          key={value}
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => choose(value)}
        >
          <GoggleIcon name="search" />
          <span>{value}</span>
          <GoggleIcon name="arrow" />
        </li>
      ))}
    </ul>
  );
}
