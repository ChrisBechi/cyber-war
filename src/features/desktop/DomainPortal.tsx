import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { ZodType } from 'zod';
import {
  domainMarketOfferSchema,
  domainRecordSchema,
  domainSearchSchema,
  domainWhoisSchema,
  emptySchema,
  request,
  subdomainSchema,
} from '../../lib/api';
import type { DomainOffer, DomainSearchResult, DomainWhois } from '../../lib/api';
import { perform, useGame } from '../../lib/game-store';
import './domain-portal.css';

const businessTypes = [
  ['company', 'Empresa'],
  ['store', 'Loja / comércio'],
  ['service', 'Serviço'],
  ['technology', 'Tecnologia'],
  ['startup', 'Startup'],
  ['app', 'Aplicativo'],
  ['organization', 'Organização'],
  ['ngo', 'ONG'],
  ['association', 'Associação'],
  ['foundation', 'Fundação'],
  ['non_profit', 'Sem fins lucrativos'],
  ['education', 'Educação'],
  ['government', 'Governo'],
  ['municipality', 'Município'],
] as const;

const statusLabels: Record<string, string> = {
  active: 'Ativo',
  expiring: 'Expirando',
  expired: 'Em carência',
  available: 'Disponível',
  restricted: 'Restrito',
  online: 'Online na Tor',
  offline: 'Offline na Tor',
};

const currency = (value: number) =>
  `$${value.toLocaleString('en-US', { maximumFractionDigits: 0 })}`;

function recordStatus(expiresAtSeconds: number, now: number) {
  if (now < expiresAtSeconds - 300) {
    return 'active';
  }
  if (now < expiresAtSeconds) {
    return 'expiring';
  }
  if (now <= expiresAtSeconds + 900) {
    return 'expired';
  }
  return 'available';
}

export function DomainPortal() {
  const world = useGame((store) => store.world);
  const busy = useGame((store) => store.busy);
  const [query, setQuery] = useState('');
  const [organization, setOrganization] = useState('');
  const [businessType, setBusinessType] = useState('company');
  const [result, setResult] = useState<DomainSearchResult | null>(null);
  const [whois, setWhois] = useState<DomainWhois | null>(null);
  const [whoisQuery, setWhoisQuery] = useState('');
  const [message, setMessage] = useState('');
  const [searching, setSearching] = useState(false);
  const [subdomainDrafts, setSubdomainDrafts] = useState<Record<string, string>>({});
  const [saleDrafts, setSaleDrafts] = useState<Record<string, string>>({});
  const [redirectDrafts, setRedirectDrafts] = useState<Record<string, string>>({});
  const [counterDrafts, setCounterDrafts] = useState<Record<string, string>>({});
  const automaticSearch = useRef(false);

  const registrations = useMemo(
    () =>
      Object.values(world?.domains.registrations ?? {}).filter(
        (record) => record.ownerKind === 'player',
      ),
    [world],
  );
  const pendingOffers = useMemo(
    () =>
      (world?.domains.offers ?? []).filter(
        (offer) =>
          offer.status === 'pending' &&
          registrations.some((record) => record.address === offer.address),
      ),
    [registrations, world],
  );

  const searchDomains = useCallback(
    async (value = query, type = businessType) => {
      const cleaned = value.trim();
      if (!cleaned) {
        setMessage('Informe um nome para pesquisar.');
        return;
      }
      setSearching(true);
      setMessage('');
      try {
        const next = await request(
          'domains_search',
          { query: cleaned, business_type: type },
          domainSearchSchema,
        );
        setResult(next);
      } catch (error) {
        setMessage(String(error));
      } finally {
        setSearching(false);
      }
    },
    [businessType, query],
  );

  useEffect(() => {
    if (!world || automaticSearch.current) {
      return;
    }
    automaticSearch.current = true;
    const initialQuery = world.nickname || 'cerqbus';
    setQuery(initialQuery);
    setOrganization(world.nickname);
    void searchDomains(initialQuery, businessType);
  }, [businessType, searchDomains, world]);

  async function showWhois(address = whoisQuery) {
    const cleaned = address.trim();
    if (!cleaned) {
      setMessage('Informe um domínio para consultar o WHOIS.');
      return;
    }
    try {
      setWhois(await request('domains_whois', { address: cleaned }, domainWhoisSchema));
      setWhoisQuery(cleaned);
      setMessage('');
    } catch (error) {
      setMessage(String(error));
    }
  }

  async function register(offer: DomainOffer) {
    if (!organization.trim()) {
      setMessage('Informe o nome da empresa ou organização antes de registrar.');
      return;
    }
    try {
      await perform(
        'domain_register',
        { address: offer.address, organization, business_type: businessType },
        domainRecordSchema,
      );
      setMessage(`${offer.address} foi registrado com sucesso.`);
    } catch (error) {
      setMessage(String(error));
    }
  }

  async function mutate(
    command: string,
    args: Record<string, unknown>,
    success: string,
    schema: ZodType<unknown> = emptySchema,
  ) {
    try {
      await perform(command, args, schema);
      setMessage(success);
    } catch (error) {
      setMessage(String(error));
    }
  }

  if (!world) {
    return <div className="domain-portal-loading">Carregando o catálogo de domínios…</div>;
  }

  return (
    <div className="domain-portal">
      <header className="domain-hero">
        <div>
          <p className="domain-kicker">MEUDOMINIO.COM.BR / REGISTRO E MERCADO</p>
          <h1>O endereço da sua operação.</h1>
          <p>
            Pesquise extensões, registre sua marca, configure subdomínios e acompanhe ofertas em um
            único painel.
          </p>
        </div>
        <div className="domain-balance">
          <span>Saldo disponível</span>
          <strong>{currency(world.money)}</strong>
        </div>
      </header>

      <section className="domain-search-panel" aria-label="Pesquisar domínios">
        <div className="domain-search-line">
          <label className="domain-query">
            <span>Pesquisar domínio</span>
            <input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter') {
                  void searchDomains();
                }
              }}
              placeholder="ex.: cerqbus, cerqbus.com.br"
            />
          </label>
          <label>
            <span>Perfil da organização</span>
            <select value={businessType} onChange={(event) => setBusinessType(event.target.value)}>
              {businessTypes.map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </label>
          <button
            className="domain-primary"
            disabled={searching}
            onClick={() => void searchDomains()}
          >
            {searching ? 'Pesquisando…' : 'Pesquisar'}
          </button>
        </div>
        <label className="domain-organization">
          <span>Empresa ou organização para o registro</span>
          <input
            value={organization}
            onChange={(event) => setOrganization(event.target.value)}
            placeholder="Nome público da organização"
          />
        </label>
      </section>

      {message && (
        <p className="domain-message" role="status">
          {message}
        </p>
      )}

      {result && (
        <section className="domain-section">
          <div className="domain-section-heading">
            <div>
              <p className="domain-kicker">RESULTADOS PARA “{result.query}”</p>
              <h2>Escolha sua extensão</h2>
            </div>
            <span className="domain-count">
              {result.results.filter((item) => item.available).length} disponíveis
            </span>
          </div>
          <div className="domain-result-grid">
            {result.results.map((offer) => (
              <article
                className={`domain-result-card${offer.recommended ? ' is-recommended' : ''}`}
                key={offer.address}
              >
                {offer.recommended && <span className="domain-recommended">Recomendado</span>}
                <div className="domain-result-topline">
                  <strong>{offer.address}</strong>
                  <span className={`domain-status status-${offer.status}`}>
                    {statusLabels[offer.status] ?? offer.status}
                  </span>
                </div>
                <p>{offer.description}</p>
                <small>{offer.recommendation}</small>
                <div className="domain-result-footer">
                  <div>
                    <strong>{offer.available ? currency(offer.price) : 'Indisponível'}</strong>
                    {offer.available && (
                      <small>/ primeiro ano · renovação {currency(offer.renewalPrice)}</small>
                    )}
                    {offer.premium && offer.available && (
                      <small className="domain-premium">Premium</small>
                    )}
                  </div>
                  {offer.available ? (
                    <button
                      disabled={busy || offer.restricted}
                      onClick={() => void register(offer)}
                    >
                      Registrar
                    </button>
                  ) : (
                    <button onClick={() => void showWhois(offer.address)}>WHOIS</button>
                  )}
                </div>
                {offer.restrictionReason && (
                  <small className="domain-restriction">{offer.restrictionReason}</small>
                )}
              </article>
            ))}
          </div>
          <div className="domain-suggestions">
            <span>Variações disponíveis</span>
            {result.suggestions.map((suggestion) => (
              <button
                key={suggestion}
                onClick={() => {
                  setQuery(suggestion);
                  void searchDomains(suggestion);
                }}
              >
                {suggestion}
              </button>
            ))}
          </div>
        </section>
      )}

      <section className="domain-section">
        <div className="domain-section-heading">
          <div>
            <p className="domain-kicker">MINHA CARTEIRA</p>
            <h2>Domínios registrados</h2>
          </div>
          <span className="domain-count">{registrations.length} endereços</span>
        </div>
        {registrations.length === 0 ? (
          <div className="domain-empty">
            Sua carteira está vazia. Pesquise um nome acima para começar.
          </div>
        ) : (
          <div className="domain-owned-list">
            {registrations.map((record) => {
              const state = recordStatus(record.expiresAtSeconds, world.playtimeSeconds);
              const redirect = redirectDrafts[record.address] ?? record.redirectTo ?? '';
              return (
                <article className="domain-owned-card" key={record.address}>
                  <div className="domain-owned-heading">
                    <div>
                      <div className="domain-result-topline">
                        <strong>{record.address}</strong>
                        <span className={`domain-status status-${state}`}>
                          {statusLabels[state]}
                        </span>
                        {record.primary && <span className="domain-primary-badge">Principal</span>}
                      </div>
                      <p>
                        {record.organization} · expira em{' '}
                        {record.expiresAtSeconds >= 4_000_000_000
                          ? 'sem prazo'
                          : `2028+${Math.floor(record.expiresAtSeconds / 3600)} ciclos`}
                      </p>
                    </div>
                    <button className="link-button" onClick={() => void showWhois(record.address)}>
                      Ver WHOIS
                    </button>
                  </div>
                  <div className="domain-owned-actions">
                    <button
                      disabled={busy}
                      onClick={() =>
                        void mutate(
                          'domain_renew',
                          { address: record.address },
                          `${record.address} renovado.`,
                          domainRecordSchema,
                        )
                      }
                    >
                      Renovar
                    </button>
                    {!record.primary && (
                      <button
                        disabled={busy}
                        onClick={() =>
                          void mutate(
                            'domain_set_primary',
                            { address: record.address },
                            `${record.address} definido como principal.`,
                          )
                        }
                      >
                        Definir principal
                      </button>
                    )}
                    {record.listedPrice ? (
                      <button
                        disabled={busy}
                        onClick={() =>
                          void mutate(
                            'domain_cancel_sale',
                            { address: record.address },
                            'Oferta de venda cancelada.',
                          )
                        }
                      >
                        Cancelar venda
                      </button>
                    ) : (
                      <>
                        <input
                          aria-label={`Preço de venda de ${record.address}`}
                          value={saleDrafts[record.address] ?? ''}
                          onChange={(event) =>
                            setSaleDrafts((old) => ({
                              ...old,
                              [record.address]: event.target.value,
                            }))
                          }
                          placeholder="Preço de venda"
                          inputMode="numeric"
                        />
                        <button
                          disabled={busy}
                          onClick={() =>
                            void mutate(
                              'domain_list_for_sale',
                              {
                                address: record.address,
                                price: Number(saleDrafts[record.address]),
                              },
                              `${record.address} listado para venda.`,
                              domainMarketOfferSchema,
                            )
                          }
                        >
                          Vender
                        </button>
                      </>
                    )}
                  </div>
                  <div className="domain-owned-tools">
                    <label>
                      Subdomínio
                      <div className="domain-inline-control">
                        <input
                          value={subdomainDrafts[record.address] ?? ''}
                          onChange={(event) =>
                            setSubdomainDrafts((old) => ({
                              ...old,
                              [record.address]: event.target.value,
                            }))
                          }
                          placeholder="app, api, mail"
                        />
                        <button
                          disabled={busy}
                          onClick={() =>
                            void mutate(
                              'domain_create_subdomain',
                              {
                                address: record.address,
                                label: subdomainDrafts[record.address] ?? '',
                              },
                              'Subdomínio criado.',
                              subdomainSchema,
                            )
                          }
                        >
                          Criar
                        </button>
                      </div>
                    </label>
                    <label>
                      Redirecionamento
                      <div className="domain-inline-control">
                        <select
                          value={redirect}
                          onChange={(event) =>
                            setRedirectDrafts((old) => ({
                              ...old,
                              [record.address]: event.target.value,
                            }))
                          }
                        >
                          <option value="">Sem redirecionamento</option>
                          {registrations
                            .filter((candidate) => candidate.address !== record.address)
                            .map((candidate) => (
                              <option key={candidate.address} value={candidate.address}>
                                {candidate.address}
                              </option>
                            ))}
                        </select>
                        <button
                          disabled={busy}
                          onClick={() =>
                            void mutate(
                              'domain_set_redirect',
                              { address: record.address, target: redirect || null },
                              redirect
                                ? 'Redirecionamento configurado.'
                                : 'Redirecionamento removido.',
                              domainRecordSchema,
                            )
                          }
                        >
                          Aplicar
                        </button>
                      </div>
                    </label>
                  </div>
                  {Object.values(record.subdomains).length > 0 && (
                    <div className="domain-subdomains">
                      <span>Subdomínios</span>
                      {Object.values(record.subdomains).map((subdomain) => (
                        <code key={subdomain.address}>{subdomain.address}</code>
                      ))}
                    </div>
                  )}
                </article>
              );
            })}
          </div>
        )}
      </section>

      {pendingOffers.length > 0 && (
        <section className="domain-section">
          <div className="domain-section-heading">
            <div>
              <p className="domain-kicker">MERCADO SECUNDÁRIO</p>
              <h2>Ofertas recebidas</h2>
            </div>
          </div>
          <div className="domain-offers">
            {pendingOffers.map((offer) => (
              <div className="domain-offer-row" key={offer.id}>
                <div>
                  <strong>{offer.address}</strong>
                  <span>
                    {offer.buyer} oferece <b>{currency(offer.amount)}</b>
                  </span>
                </div>
                <div className="domain-inline-control">
                  <input
                    value={counterDrafts[offer.id] ?? ''}
                    onChange={(event) =>
                      setCounterDrafts((old) => ({ ...old, [offer.id]: event.target.value }))
                    }
                    placeholder="Contraproposta"
                    inputMode="numeric"
                  />
                  <button
                    disabled={busy}
                    onClick={() =>
                      void mutate(
                        'domain_offer_respond',
                        { offer_id: offer.id, action: 'accept', counter_price: null },
                        'Oferta aceita.',
                      )
                    }
                  >
                    Aceitar
                  </button>
                  <button
                    disabled={busy}
                    onClick={() =>
                      void mutate(
                        'domain_offer_respond',
                        {
                          offer_id: offer.id,
                          action: 'counter',
                          counter_price: Number(counterDrafts[offer.id]),
                        },
                        'Contraproposta enviada.',
                      )
                    }
                  >
                    Contrapor
                  </button>
                  <button
                    disabled={busy}
                    onClick={() =>
                      void mutate(
                        'domain_offer_respond',
                        { offer_id: offer.id, action: 'reject', counter_price: null },
                        'Oferta recusada.',
                      )
                    }
                  >
                    Recusar
                  </button>
                </div>
              </div>
            ))}
          </div>
        </section>
      )}

      <section className="domain-section domain-whois-section">
        <div className="domain-section-heading">
          <div>
            <p className="domain-kicker">CONSULTA PÚBLICA</p>
            <h2>WHOIS simplificado</h2>
          </div>
        </div>
        <div className="domain-whois-search">
          <input
            value={whoisQuery}
            onChange={(event) => setWhoisQuery(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter') {
                void showWhois();
              }
            }}
            placeholder="dominio.com.br"
          />
          <button onClick={() => void showWhois()}>Consultar</button>
        </div>
        {whois && (
          <div className="domain-whois-card">
            <div>
              <strong>{whois.address}</strong>
              <span className={`domain-status status-${whois.status}`}>
                {statusLabels[whois.status] ?? whois.status}
              </span>
            </div>
            <dl>
              <div>
                <dt>IP associado</dt>
                <dd>
                  {whois.network === 'tor'
                    ? 'Não publicado (rede Tor)'
                    : (whois.ip ?? 'Ainda sem hospedagem')}
                </dd>
              </div>
              <div>
                <dt>Organização</dt>
                <dd>{whois.owner ?? '—'}</dd>
              </div>
              <div>
                <dt>Extensão</dt>
                <dd>
                  {whois.suffix} · {whois.category}
                </dd>
              </div>
              {whois.network === 'tor' && (
                <div>
                  <dt>Classificação</dt>
                  <dd>TOR · Onion Service v{whois.addressVersion ?? 3} · sem DNS tradicional</dd>
                </div>
              )}
              <div>
                <dt>Redireciona para</dt>
                <dd>{whois.redirectTo ?? '—'}</dd>
              </div>
              <div>
                <dt>Subdomínios</dt>
                <dd>{whois.subdomains.length ? whois.subdomains.join(', ') : '—'}</dd>
              </div>
            </dl>
          </div>
        )}
      </section>
    </div>
  );
}
