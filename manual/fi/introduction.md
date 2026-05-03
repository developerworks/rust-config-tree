# Johdanto

[English](../en/introduction.html) | [中文](../zh/introduction.html) | [日本語](../ja/introduction.html) | [한국어](../ko/introduction.html) | [Français](../fr/introduction.html) | [Deutsch](../de/introduction.html) | [Español](../es/introduction.html) | [Português](../pt/introduction.html) | [Svenska](../sv/introduction.html) | [Suomi](introduction.html) | [Nederlands](../nl/introduction.html)

`rust-config-tree` tarjoaa uudelleenkaytettavan konfiguraatiopuun latauksen ja CLI-apurit Rust-sovelluksille, jotka kayttavat kerrostettuja konfiguraatiotiedostoja.

Crate on suunniteltu pienen vastuunjaon ymparille:

- `confique` omistaa skeemamaaritykset, koodin oletusarvot, validoinnin ja konfiguraatiomallien luonnin.
- `figment` omistaa runtime-latauksen ja runtime-lahdemetadatan.
- `rust-config-tree` omistaa rekursiivisen include-lapikaynnin, include-polkujen ratkaisun, `.env`-latauksen, mallikohteiden tunnistuksen ja uudelleenkaytettavat clap-komennot.

Crate on hyodyllinen, kun sovellus haluaa luonnollisen konfiguraatiotiedostojen asettelun, kuten taman:

```yaml
include:
  - config/server.yaml
  - config/database.yaml

log:
  level: info
```

Jokainen sisallytetty tiedosto voi kayttaa samaa skeemamuotoa, ja suhteelliset include-polut ratkaistaan ne maaritelleesta tiedostosta. Lopullinen konfiguraatio on silti tavallinen `confique`-skeema-arvo.

## Paatoiminnot

- Rekursiivinen include-lapikaynti syklien tunnistuksella.
- Suhteelliset include-polut ratkaistaan maarittelevasta tiedostosta.
- `.env` ladataan ennen ymparistoprovidereiden arviointia.
- Skeemassa maaritellyt ymparistomuuttujat ilman erotinmerkkijakoa.
- Figment-metadata runtime-lahteen seurantaan.
- TRACE-tason lahteenseurannan tapahtumat `tracing`-kirjaston kautta.
- Draft 7 JSON Schema -luonti editorien taydennysta ja validointia varten.
- YAML-, TOML-, JSON- ja JSON5-mallien luonti.
- TOML `#:schema`- ja YAML Language Server -skeemamallirivit luoduille malleille.
- Opt-in YAML-mallien jakaminen `x-tree-split`-merkityille osioille.
- Sisaanrakennetut clap-alikomennot konfiguraatiomalleille, JSON Schemalle ja shell-taydennyksille.
- Alemman tason puu-API kutsujille, jotka eivat kayta `confique`-kirjastoa.

## Julkiset aloituspisteet

Kayta naita APIeja useimmissa sovelluksissa:

- `load_config::<S>(path)` lataa lopullisen skeeman.
- `load_config_with_figment::<S>(path)` lataa skeeman ja palauttaa lahteen seurantaan kaytetyn Figment-graafin.
- `write_config_templates::<S>(config_path, output_path)` kirjoittaa juurimallin ja rekursiivisesti loydetyt lapsimallit.
- `write_config_schemas::<S>(output_path)` kirjoittaa juuri- ja osio-Draft 7 JSON Schema -skeemat.
- `handle_config_command::<Cli, S>(command, config_path)` kasittelee sisaanrakennetut clap-konfiguraatiokomennot.

Kayta `load_config_tree`-funktiota, kun tarvitset lapikayntiprimitiivin ilman `confique`-kirjastoa.
