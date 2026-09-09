/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Origine du relais météo. Absente : appels directs aux fournisseurs. */
  readonly VITE_KLIMA_RELAY?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
