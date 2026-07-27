# Component 2: `offline-db-sync`

Este componente contiene la especificación e implementación del módulo de sincronización offline:
1. Endpoint masivo en el servidor de Rust (`GET /api/productos/sync`).
2. Almacenamiento local nativo en `IndexedDB` en el cliente Svelte.

## Estructura
- Backend: Modificaciones en `src/api.rs` y `src/db.rs`.
- Frontend: Módulo `IndexedDB` en `frontend/src/App.svelte`.

## Prompt de Ejecución para Agente:
Ver documento `implementation_plan.md` en la raíz del proyecto.
