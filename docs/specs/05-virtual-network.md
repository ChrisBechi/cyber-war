# Spec 05 — Virtual Network

## Objetivo

Criar internet e LANs totalmente fictícias.

## Entidades

`VirtualNetwork`, `VirtualSubnet`, `VirtualRouter`, `VirtualWifi`, `VirtualHost`, `VirtualService`, `VirtualUser`, `VirtualCredential`, `VirtualFirewall`, `VirtualDnsRecord`, `VirtualDevice`.

## Regra

`nmap 10.20.4.15` consulta um VirtualHost.

`ssh user@10.20.4.15` cria VirtualSession.

`curl https://target.local` consulta VirtualService.

Nada gera tráfego real.

## Wireless

Suportar SSID, BSSID, canal, força de sinal simulada, perfil de criptografia, clients e access state.

## Privilégios

```text
WIRELESS
→ HOST ACCESS
→ USER
→ ADMIN/ROOT
→ OBJECTIVE
```

Nem toda missão usa todas as etapas.

## Reação persistente

Hosts podem ter serviço derrubado, senha alterada, firewall modificado, falha corrigida, host removido ou evidência gerada.
