import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { sharedMessages, useRelay } from '@klima/core'
import { I18nProvider } from '@klima/core/ui'
import App from './App'
import { webMessages } from './i18n/messages'
import './index.css'

const CATALOGS = [sharedMessages, webMessages]

/**
 * L'acheminement des appels météo.
 *
 * Sans relais configuré, chaque navigateur interroge les fournisseurs
 * lui-même : c'est le mode de développement, et il reste sur le plan gratuit
 * d'Open-Meteo, réservé à un usage non commercial. Avec un relais, les appels
 * passent par lui — clé commerciale, cache mutualisé, et MET Norway devient
 * accessible au web puisque c'est le serveur qui se nomme.
 */
const RELAY = import.meta.env.VITE_KLIMA_RELAY
if (RELAY) useRelay(RELAY)

const container = document.getElementById('root')
if (!container) throw new Error('Élément #root introuvable dans index.html')

createRoot(container).render(
  <StrictMode>
    <I18nProvider catalogs={CATALOGS}>
      <App />
    </I18nProvider>
  </StrictMode>,
)
