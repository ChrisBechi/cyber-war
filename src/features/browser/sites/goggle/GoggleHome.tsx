import { GoggleLogo } from './GoggleLogo';
import { SearchBox } from './SearchBox';

export function GoggleHome({
  search,
  imageSearch,
}: {
  search: (query: string, lucky?: boolean) => void;
  imageSearch: (source: string) => void;
}) {
  return (
    <main className="goggle-home">
      <div className="goggle-home-center">
        <h1>
          <GoggleLogo />
        </h1>
        <SearchBox search={search} imageSearch={imageSearch} />
      </div>
    </main>
  );
}
