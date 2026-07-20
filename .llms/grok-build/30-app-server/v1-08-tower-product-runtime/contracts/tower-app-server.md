# Contrato App Server — integração com `tower`

**Fonte de verdade local do epic.** O supervisor chama o listener WS com
`bind: SocketAddr` e `bearer_token: String`, recebe `WsListenerHandle` e não
acessa `FacadeProcessor` internamente.

O listener deve bindar antes de ser publicado como pronto; falha de bind é
propagada. O supervisor é responsável por abortar o handle em rollback e
shutdown. Auth é obrigatória por padrão; `--insecure-no-auth` permite o modo
explícito sem bearer para integrações locais. O endpoint de health funcional é
o `initialize` do protocolo.
