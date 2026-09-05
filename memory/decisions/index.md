# Decisiones de arquitectura (ADR)

Un archivo por decisión, numerado en orden de creación. Una vez aceptado, un ADR
**no se edita** — si la decisión cambia, se crea un ADR nuevo que la referencia y la
marca como `Superseded by 000X` en la tabla de abajo. Esto es intencional: el valor de
un ADR es ser la foto exacta de qué se decidió y por qué en ese momento, no un
documento vivo (para eso está `../STATUS.md`).

Antes de tocar algo que ya tiene un ADR, léelo primero — evita reabrir un trade-off
ya discutido y aceptado explícitamente.

| # | Título | Estado |
|---|--------|--------|
| [0001](0001-single-active-install-scope.md) | Un solo scope de instalación activo (Project o Global), no por agente | Accepted |
| [0002](0002-no-global-state-library.md) | Sin librería de estado global (Pinia); store reactivo de módulo | Accepted |
| [0003](0003-generated-artifacts-consolidated.md) | Todo lo generado bajo `.generated/`, excepto `node_modules/` y `target/` | Accepted |
